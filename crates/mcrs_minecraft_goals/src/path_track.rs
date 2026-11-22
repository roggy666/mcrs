use alloc::boxed::Box;
use core::hash::Hash;
use core::hash::Hasher;
use core::mem;

use glam::DVec3;
use mcrs_minecraft_brain::Candidate;
use mcrs_minecraft_brain::Intents;
use mcrs_minecraft_brain::IssuedPath;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_brain::PathIntent;
use mcrs_minecraft_brain::Progress;
use mcrs_minecraft_brain::SearchSlice;
use mcrs_minecraft_brain::Ticket;
use mcrs_minecraft_brain::Turn;
use mcrs_minecraft_brain::TurnScratch;
use mcrs_minecraft_pathfind::FoundPath;
use mcrs_minecraft_pathfind::MobPath;
use mcrs_minecraft_pathfind::PathSearchWorld;
use mcrs_minecraft_pathfind::Request;
use mcrs_minecraft_pathfind::RequestKind;
use mcrs_minecraft_pathfind::SearchStep;
use mcrs_minecraft_pathfind::SearchWorld;
use mcrs_minecraft_pathfind::adjust_target;
use mcrs_minecraft_pathfind::can_update_path;
use mcrs_minecraft_pathfind::search_advance;
use mcrs_minecraft_pathfind::search_begin;
use mcrs_voxel::BlockPos;

const RECOMPUTE_GATE: u64 = 20;
const LEAST_LENGTH: f32 = 16.0;
const STEP_HEIGHT: f32 = 0.6;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PathTrack {
    issued: Ticket,
    route: Route,
    requested: Option<BlockPos>,
    pending: Option<Box<Pending>>,
    stored: Option<Box<Stored>>,
    last_recompute: u64,
    found: Option<Box<Found>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Route {
    #[default]
    None,
    Walking,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Mode {
    Walk,
    Check,
}

#[derive(Clone, Debug, PartialEq)]
struct Pending {
    request: Request,
    speed: f32,
    mode: Mode,
}

#[derive(Clone, Debug, PartialEq)]
struct Stored {
    request: Request,
    speed: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct Found {
    tick: u64,
    target: BlockPos,
    path: FoundPath,
}

enum Run {
    Found(FoundPath),
    NoPath,
    Deferred,
}

impl Hash for PathTrack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.issued.hash(state);
        self.route.hash(state);
        self.requested.hash(state);
        self.pending
            .as_ref()
            .map(|pending| (&pending.request.targets, pending.mode))
            .hash(state);
        self.last_recompute.hash(state);
    }
}

fn mob_path<W: ?Sized>(turn: &Turn<'_, W>) -> MobPath {
    let view = turn.view;
    let dimensions = turn.kinds.dimensions(view);
    let mut mob = MobPath::standing(
        view.position,
        dimensions.width,
        dimensions.height,
        STEP_HEIGHT,
    );
    mob.on_ground = view.flags.contains(MobFlags::ON_GROUND);
    mob.in_liquid = view.flags.contains(MobFlags::IN_WATER);
    if let Some(kind) = turn.kinds.get(view.kind) {
        for (path_type, malus) in kind.malus {
            mob.malus.set(*path_type, *malus);
        }
    }
    mob
}

fn length(view: &MobView) -> f32 {
    view.follow_range.max(LEAST_LENGTH)
}

fn run<W: PathSearchWorld + ?Sized>(
    turn: &Turn<'_, W>,
    scratch: &mut TurnScratch,
    request: &Request,
) -> Run {
    let mob = mob_path(turn);
    if !can_update_path(mob.on_ground, mob.in_liquid, false) {
        return Run::NoPath;
    }
    let mut request = request.clone();
    if request.targets.len() == 1 {
        let Some(&target) = request.targets.first() else {
            return Run::NoPath;
        };
        if !turn.world.column_loaded(target) {
            return Run::NoPath;
        }
        request.targets = alloc::vec![adjust_target(turn.world, target)];
    }
    if let Some(slice) = scratch.slice.as_deref()
        && slice.key == turn.view.key
    {
        if slice.request == request {
            return resume(turn, scratch);
        }
        if let Some(slice) = scratch.slice.take() {
            scratch.searcher = slice.searcher;
        }
    }
    if scratch.budget == 0 {
        scratch.deferred = true;
        return Run::Deferred;
    }
    let version = turn.world.version();
    let mut world = turn.world.open(&mob);
    if !search_begin(&mut world, &mob, &request, &mut scratch.searcher) {
        return Run::NoPath;
    }
    advance(turn, scratch, &mut world, mob, request, version)
}

fn advance<W: PathSearchWorld + ?Sized, S: SearchWorld>(
    turn: &Turn<'_, W>,
    scratch: &mut TurnScratch,
    world: &mut S,
    mob: MobPath,
    request: Request,
    version: u64,
) -> Run {
    match search_advance(world, &mob, &request, &mut scratch.searcher, scratch.budget) {
        SearchStep::Done { outcome, spent } => {
            scratch.budget = scratch.budget.saturating_sub(spent.max(1));
            outcome.path.map_or(Run::NoPath, Run::Found)
        }
        SearchStep::Suspended { spent } => {
            scratch.budget = scratch.budget.saturating_sub(spent);
            scratch.deferred = true;
            if scratch.slice.is_none() {
                let searcher = mem::take(&mut scratch.searcher);
                scratch.slice = Some(Box::new(SearchSlice {
                    key: turn.view.key,
                    version,
                    mob,
                    request,
                    searcher,
                }));
            } else {
                scratch.searcher.clear();
            }
            Run::Deferred
        }
    }
}

fn resume<W: PathSearchWorld + ?Sized>(turn: &Turn<'_, W>, scratch: &mut TurnScratch) -> Run {
    let Some(mut slice) = scratch.slice.take() else {
        return Run::NoPath;
    };
    if scratch.budget == 0 {
        scratch.slice = Some(slice);
        scratch.deferred = true;
        return Run::Deferred;
    }
    let Some(mut world) = turn.world.reopen(&slice.mob, slice.version) else {
        scratch.searcher = slice.searcher;
        scratch.searcher.clear();
        let version = turn.world.version();
        let mut world = turn.world.open(&slice.mob);
        if !search_begin(
            &mut world,
            &slice.mob,
            &slice.request,
            &mut scratch.searcher,
        ) {
            return Run::NoPath;
        }
        return advance(turn, scratch, &mut world, slice.mob, slice.request, version);
    };
    match search_advance(
        &mut world,
        &slice.mob,
        &slice.request,
        &mut slice.searcher,
        scratch.budget,
    ) {
        SearchStep::Done { outcome, spent } => {
            scratch.budget = scratch.budget.saturating_sub(spent.max(1));
            scratch.searcher = slice.searcher;
            outcome.path.map_or(Run::NoPath, Run::Found)
        }
        SearchStep::Suspended { spent } => {
            scratch.budget = scratch.budget.saturating_sub(spent);
            scratch.deferred = true;
            scratch.slice = Some(slice);
            Run::Deferred
        }
    }
}

impl PathTrack {
    #[must_use]
    pub fn is_running(&self, view: &MobView) -> bool {
        match self.route {
            Route::None => false,
            Route::Walking => {
                let applied = view.path();
                applied.ticket != self.issued || applied.progress == Progress::Running
            }
        }
    }

    pub fn request<W: PathSearchWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        point: DVec3,
        speed: f32,
        intents: &mut Intents,
    ) {
        let target = BlockPos::from(point);
        if self.same_target(turn.view, target, speed, intents) {
            return;
        }
        let request = Request::to_point(point, speed, length(turn.view));
        self.ask(turn, scratch, request, speed, intents);
    }

    pub fn request_to<W: PathSearchWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        candidate: &Candidate,
        speed: f32,
        intents: &mut Intents,
    ) -> bool {
        let target = BlockPos::from(candidate.position);
        if self.same_target(turn.view, target, speed, intents) {
            return true;
        }
        if let Some(found) = self
            .found
            .take()
            .filter(|found| found.tick == turn.tick && found.target == target)
        {
            self.walk(turn, scratch, found.path, target, speed, intents);
            return true;
        }
        let request = Request::to_entity(candidate.position, speed, length(turn.view));
        self.ask(turn, scratch, request, speed, intents)
    }

    pub fn has_route_to<W: PathSearchWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        candidate: &Candidate,
    ) -> bool {
        self.check(
            turn,
            scratch,
            Request::to_entity(candidate.position, 1.0, length(turn.view)),
        )
    }

    pub fn has_route_to_point<W: PathSearchWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        point: DVec3,
    ) -> bool {
        self.check(
            turn,
            scratch,
            Request::to_point(point, 1.0, length(turn.view)),
        )
    }

    pub fn clear(&mut self, intents: &mut Intents) {
        self.issued = self.issued.next();
        self.route = Route::None;
        self.requested = None;
        self.stored = None;
        intents.path = Some(PathIntent::Clear {
            ticket: self.issued,
        });
    }

    pub fn retry<W: PathSearchWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        intents: &mut Intents,
    ) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let pending = *pending;
        let target = pending.request.targets.first().copied();
        match run(turn, scratch, &pending.request) {
            Run::Found(found) => match pending.mode {
                Mode::Walk => {
                    let target = target.unwrap_or(found.target);
                    self.walk(turn, scratch, found, target, pending.speed, intents);
                }
                Mode::Check => {
                    if let Some(target) = target {
                        self.found = Some(Box::new(Found {
                            tick: turn.tick,
                            target,
                            path: found,
                        }));
                    }
                }
            },
            Run::NoPath => {
                if pending.mode == Mode::Walk && pending.request.kind != RequestKind::Entity {
                    self.clear(intents);
                }
            }
            Run::Deferred => self.pending = Some(Box::new(pending)),
        }
    }

    pub fn recompute_if_due<W: PathSearchWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        intents: &mut Intents,
    ) {
        if !turn.view.flags.contains(MobFlags::RECOMPUTE_PATH)
            || !self.is_running(turn.view)
            || turn.tick.saturating_sub(self.last_recompute) <= RECOMPUTE_GATE
        {
            return;
        }
        let Some(stored) = self.stored.clone() else {
            return;
        };
        self.last_recompute = turn.tick;
        let mut request = stored.request.clone();
        request.kind = RequestKind::Blocks;
        self.ask(turn, scratch, request, stored.speed, intents);
    }

    fn same_target(
        &self,
        view: &MobView,
        target: BlockPos,
        speed: f32,
        intents: &mut Intents,
    ) -> bool {
        if self.is_running(view) && self.requested == Some(target) {
            intents.speed = Some(speed);
            return true;
        }
        false
    }

    fn ask<W: PathSearchWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        request: Request,
        speed: f32,
        intents: &mut Intents,
    ) -> bool {
        let target = request.targets.first().copied();
        match run(turn, scratch, &request) {
            Run::Found(found) => {
                let target = target.unwrap_or(found.target);
                self.stored = Some(Box::new(Stored {
                    request: request.clone(),
                    speed,
                }));
                self.walk(turn, scratch, found, target, speed, intents);
                true
            }
            Run::NoPath => {
                if request.kind != RequestKind::Entity {
                    self.clear(intents);
                }
                false
            }
            Run::Deferred => {
                self.pending = Some(Box::new(Pending {
                    request,
                    speed,
                    mode: Mode::Walk,
                }));
                true
            }
        }
    }

    fn check<W: PathSearchWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        request: Request,
    ) -> bool {
        let Some(target) = request.targets.first().copied() else {
            return false;
        };
        if let Some(found) = &self.found
            && found.tick == turn.tick
            && found.target == target
        {
            return found.path.reached;
        }
        match run(turn, scratch, &request) {
            Run::Found(found) => {
                let reached = found.reached;
                self.found = Some(Box::new(Found {
                    tick: turn.tick,
                    target,
                    path: found,
                }));
                reached
            }
            Run::NoPath => false,
            Run::Deferred => {
                self.pending = Some(Box::new(Pending {
                    request,
                    speed: 1.0,
                    mode: Mode::Check,
                }));
                false
            }
        }
    }

    fn walk<W: ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        found: FoundPath,
        target: BlockPos,
        speed: f32,
        intents: &mut Intents,
    ) {
        self.issued = self.issued.next();
        self.route = Route::Walking;
        self.requested = Some(target);
        intents.path = Some(PathIntent::Walk {
            ticket: self.issued,
            target: found.target.bottom_centre(),
            speed_modifier: speed,
        });
        scratch.paths.push(IssuedPath {
            mob: turn.view.mob,
            ticket: self.issued,
            path: found,
        });
    }
}
