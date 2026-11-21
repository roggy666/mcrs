use glam::DVec3;
use mcrs_minecraft_brain::Candidate;
use mcrs_minecraft_brain::CandidateFlags;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::Intents;
use mcrs_minecraft_brain::KindFlags;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::LookIntent;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_brain::TargetIntent;
use mcrs_minecraft_brain::Turn;
use mcrs_minecraft_brain::TurnScratch;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_voxel::BlockPos;

use crate::PathTrack;
use crate::look_hold;

const CANDIDATE_MARGIN: f64 = 2.0;
const MIN_REACH: f64 = 2.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Wake {
    pub(crate) busy: bool,
    pub(crate) changed: bool,
    pub(crate) due: Option<u64>,
    pub(crate) draws: bool,
}

impl Wake {
    pub(crate) fn due_at(&mut self, tick: Option<u64>) {
        if let Some(tick) = tick {
            self.due = Some(self.due.map_or(tick, |due| due.min(tick)));
        }
    }
}

pub(crate) struct Cx<'r, 't, 'a, W: ?Sized> {
    pub(crate) turn: &'t Turn<'a, W>,
    pub(crate) scratch: &'r mut TurnScratch,
    pub(crate) intents: &'r mut Intents,
    pub(crate) path: &'r mut PathTrack,
    pub(crate) target: &'r mut EntityRef,
    pub(crate) wake: &'r mut Wake,
    pub(crate) destination: Option<DVec3>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ray {
    Check,
    Skip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Visibility {
    Use,
    Ignore,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Combat {
    pub(crate) range: f64,
    pub(crate) ray: Ray,
    pub(crate) visibility: Visibility,
}

impl<W: GoalWorld + ?Sized> Cx<'_, '_, '_, W> {
    pub(crate) fn me(&self) -> &MobView {
        self.turn.view
    }

    pub(crate) fn tick(&self) -> u64 {
        self.turn.tick
    }

    pub(crate) fn eye(&self) -> DVec3 {
        let dimensions = self.turn.kinds.dimensions(self.turn.view);
        self.turn.view.eye(dimensions.eye_height)
    }

    pub(crate) fn pitch_limit(&self) -> f32 {
        self.turn
            .kinds
            .get(self.turn.view.kind)
            .map_or(40.0, |data| data.pitch_limit)
    }

    pub(crate) fn head_yaw_limit(&self) -> f32 {
        self.turn
            .kinds
            .get(self.turn.view.kind)
            .map_or(75.0, |data| data.head_yaw_limit)
    }

    pub(crate) fn tempt_range(&self) -> f64 {
        f64::from(
            self.turn
                .kinds
                .get(self.turn.view.kind)
                .map_or(10.0, |data| data.tempt_range),
        )
    }

    pub(crate) fn follow_range(&self) -> f64 {
        f64::from(self.turn.view.follow_range)
    }

    pub(crate) fn set_target(&mut self, target: Option<EntityId>) {
        let next = EntityRef::some(target);
        if *self.target != next {
            *self.target = next;
            self.intents.target = Some(match target {
                Some(target) => TargetIntent::Set(target),
                None => TargetIntent::Clear,
            });
        }
    }

    pub(crate) fn attack_target(&self) -> Option<Candidate> {
        let target = self.target.get()?;
        let candidate = self.find(target)?;
        self.can_attack(&candidate).then_some(candidate)
    }

    pub(crate) fn find(&self, id: EntityId) -> Option<Candidate> {
        if let Some(view) = self.turn.neighbours.find(id) {
            return Some(Candidate::mob(view, self.turn.kinds.dimensions(view)));
        }
        self.turn.table.find(id).map(Candidate::player)
    }

    pub(crate) fn can_attack(&self, candidate: &Candidate) -> bool {
        let me = self.turn.view;
        if candidate.id == me.mob || !candidate.is_alive() {
            return false;
        }
        if candidate.flags.contains(CandidateFlags::SPECTATOR)
            || candidate.flags.contains(CandidateFlags::INVULNERABLE)
        {
            return false;
        }
        if candidate.is_player() && candidate.flags.contains(CandidateFlags::PEACEFUL) {
            return false;
        }
        if candidate
            .kind
            .is_some_and(|kind| self.turn.kinds.has(kind, KindFlags::UNTARGETABLE))
        {
            return false;
        }
        if me.flags.contains(MobFlags::TAME) && me.owner.is(candidate.id) {
            return false;
        }
        true
    }

    pub(crate) fn combat_test(&self, candidate: &Candidate, combat: Combat) -> bool {
        let me = self.turn.view;
        if candidate.id == me.mob
            || !candidate.is_alive()
            || candidate.flags.contains(CandidateFlags::SPECTATOR)
        {
            return false;
        }
        if !self.can_attack(candidate) {
            return false;
        }
        self.in_range_and_sight(candidate, combat)
    }

    pub(crate) fn in_range_and_sight(&self, candidate: &Candidate, combat: Combat) -> bool {
        if combat.range > 0.0 {
            let factor = match combat.visibility {
                Visibility::Use => f64::from(candidate.visibility),
                Visibility::Ignore => 1.0,
            };
            let reach = (combat.range * factor).max(MIN_REACH);
            if candidate.position.distance_squared(self.turn.view.position) > reach * reach {
                return false;
            }
        }
        match combat.ray {
            Ray::Check => self.sees(candidate),
            Ray::Skip => true,
        }
    }

    pub(crate) fn sees(&self, candidate: &Candidate) -> bool {
        self.turn.world.line_of_sight(self.eye(), candidate.eye())
    }

    pub(crate) fn feet_distance_squared(&self, candidate: &Candidate) -> f64 {
        candidate.position.distance_squared(self.turn.view.position)
    }

    pub(crate) fn select_distance(&self, candidate: &Candidate) -> f64 {
        let eye = self.eye();
        candidate.position.distance_squared(eye)
    }

    pub(crate) fn grown_box(&self, grow_xz: f64, grow_y: f64) -> (DVec3, DVec3) {
        let me = self.turn.view;
        let dimensions = self.turn.kinds.dimensions(me);
        let half = f64::from(dimensions.width) * 0.5;
        let min = DVec3::new(
            me.position.x - half - grow_xz,
            me.position.y - grow_y,
            me.position.z - half - grow_xz,
        );
        let max = DVec3::new(
            me.position.x + half + grow_xz,
            me.position.y + f64::from(dimensions.height) + grow_y,
            me.position.z + half + grow_xz,
        );
        (min, max)
    }

    pub(crate) fn collect_mobs(&mut self, min: DVec3, max: DVec3, kinds: &KindSet) {
        let kinds_table = self.turn.kinds;
        let me = self.turn.view.mob;
        let grown_min = min - DVec3::splat(CANDIDATE_MARGIN);
        let grown_max = max + DVec3::splat(CANDIDATE_MARGIN);
        let out = &mut self.scratch.candidates;
        out.clear();
        self.turn
            .neighbours
            .for_each_in(grown_min, grown_max, kinds, |view| {
                if view.mob == me {
                    return;
                }
                let candidate = Candidate::mob(view, kinds_table.dimensions(view));
                if candidate.intersects(min, max) {
                    out.push(candidate);
                }
            });
        out.sort_by_key(|candidate| candidate.key);
    }

    pub(crate) fn collect_players(&mut self) {
        let out = &mut self.scratch.candidates;
        out.clear();
        out.extend(self.turn.players.iter().map(Candidate::player));
    }

    pub(crate) fn nearest(
        &mut self,
        distance: impl Fn(&Self, &Candidate) -> f64,
        accept: impl Fn(&Self, &Candidate) -> bool,
    ) -> Option<Candidate> {
        let mut best: Option<(f64, Candidate)> = None;
        let candidates = core::mem::take(&mut self.scratch.candidates);
        for candidate in &candidates {
            let value = distance(self, candidate);
            if best.is_some_and(|(best_value, _)| value >= best_value) {
                continue;
            }
            if accept(self, candidate) {
                best = Some((value, *candidate));
            }
        }
        self.scratch.candidates = candidates;
        best.map(|(_, candidate)| candidate)
    }

    pub(crate) fn nearest_sorted(
        &mut self,
        distance: impl Fn(&Self, &Candidate) -> f64,
        accept: impl Fn(&Self, &Candidate) -> bool,
    ) -> Option<Candidate> {
        let mut candidates = core::mem::take(&mut self.scratch.candidates);
        let mut ranked: alloc::vec::Vec<(f64, Candidate)> = candidates
            .iter()
            .map(|candidate| (distance(self, candidate), *candidate))
            .collect();
        ranked.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.key.cmp(&b.1.key)));
        let found = ranked
            .into_iter()
            .find(|(_, candidate)| accept(self, candidate))
            .map(|(_, candidate)| candidate);
        candidates.clear();
        self.scratch.candidates = candidates;
        found
    }

    pub(crate) fn look_at(
        &mut self,
        point: DVec3,
        turn_speed: Option<f32>,
        pitch_limit: Option<f32>,
    ) {
        self.intents.look = Some(LookIntent::At {
            point,
            turn_speed,
            pitch_limit,
            hold: look_hold(self.turn.max_lag),
        });
    }

    pub(crate) fn request_path(&mut self, point: DVec3, speed_modifier: f32) {
        self.path
            .request(self.turn, point, speed_modifier, self.intents);
    }

    pub(crate) fn request_path_to(&mut self, candidate: &Candidate, speed_modifier: f32) -> bool {
        let target = BlockPos::from(candidate.position);
        match self.turn.world.route(self.turn.view.position, target) {
            Some(end) => {
                self.path.walk(end, speed_modifier, self.intents);
                true
            }
            None => false,
        }
    }

    pub(crate) fn has_route_to(&self, candidate: &Candidate) -> bool {
        let target = BlockPos::from(candidate.position);
        self.turn
            .world
            .route(self.turn.view.position, target)
            .is_some()
    }

    pub(crate) fn clear_path(&mut self) {
        self.path.clear(self.intents);
    }

    pub(crate) fn path_running(&self) -> bool {
        self.path.is_running(self.turn.view)
    }

    pub(crate) fn busy(&mut self) {
        self.wake.busy = true;
    }
}
