use alloc::vec::Vec;
use core::hash::Hash;
use core::hash::Hasher;

use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::GoalSpec;
use mcrs_minecraft_brain::Goals;
use mcrs_minecraft_brain::Intents;
use mcrs_minecraft_brain::MobBrain;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::Panic;
use mcrs_minecraft_brain::Selector;
use mcrs_minecraft_brain::Turn;
use mcrs_minecraft_brain::TurnScratch;
use mcrs_minecraft_goal_world::GoalWorld;

use crate::GoalKind;
use crate::GoalRow;
use crate::GoalState;
use crate::GoalTable;
use crate::PathTrack;
use crate::avoid;
use crate::avoid::AvoidParams;
use crate::breed;
use crate::eat_grass;
use crate::follow_adult;
use crate::hurt_by;
use crate::leap;
use crate::melee;
use crate::nearest_target;
use crate::nearest_target::NearestParams;
use crate::owner;
use crate::panic;
use crate::query::Cx;
use crate::query::Wake;
use crate::ranged;
use crate::ranged::RangedParams;
use crate::tempt;

#[derive(Clone, Debug)]
pub struct TableBrain {
    table: &'static GoalTable,
    targets: Selector,
    goals: Selector,
    state: Vec<GoalState>,
    path: PathTrack,
    published: Panic,
}

impl PartialEq for TableBrain {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.table, other.table)
            && self.targets == other.targets
            && self.goals == other.goals
            && self.state == other.state
            && self.path == other.path
            && self.published == other.published
    }
}

impl Eq for TableBrain {}

impl Hash for TableBrain {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (core::ptr::from_ref(self.table) as usize).hash(state);
        self.targets.hash(state);
        self.goals.hash(state);
        self.state.hash(state);
        self.path.hash(state);
        self.published.hash(state);
    }
}

impl TableBrain {
    #[must_use]
    pub fn new(table: &'static GoalTable) -> Self {
        let state = table
            .targets
            .iter()
            .chain(table.goals)
            .map(|row| initial_state(&row.goal))
            .collect();
        Self {
            table,
            targets: Selector::default(),
            goals: Selector::default(),
            state,
            path: PathTrack::default(),
            published: Panic::Calm,
        }
    }

    #[must_use]
    pub const fn table(&self) -> &'static GoalTable {
        self.table
    }

    #[must_use]
    pub const fn selector(&self) -> &Selector {
        &self.goals
    }

    #[must_use]
    pub const fn target_selector(&self) -> &Selector {
        &self.targets
    }

    #[must_use]
    pub fn runs(&self, test: impl Fn(&GoalKind) -> bool) -> bool {
        self.table
            .goals
            .iter()
            .enumerate()
            .any(|(goal, row)| self.goals.is_running(goal) && test(&row.goal))
    }

    #[must_use]
    pub fn runs_target(&self, test: impl Fn(&GoalKind) -> bool) -> bool {
        self.table
            .targets
            .iter()
            .enumerate()
            .any(|(goal, row)| self.targets.is_running(goal) && test(&row.goal))
    }

    #[must_use]
    pub fn panics(&self) -> bool {
        self.runs(|goal| matches!(goal, GoalKind::Panic { .. }))
    }

    #[must_use]
    pub fn goal_state(&self, goal: usize) -> Option<&GoalState> {
        self.state
            .get(self.table.targets.len().saturating_add(goal))
    }
}

fn initial_state(kind: &GoalKind) -> GoalState {
    match kind {
        GoalKind::Breed { .. } => GoalState::Breed(Default::default()),
        GoalKind::Tempt { .. } => GoalState::Tempt(Default::default()),
        GoalKind::FollowAdult { .. } => GoalState::Follow(Default::default()),
        GoalKind::EatGrass => GoalState::Eat(Default::default()),
        GoalKind::Stroll { .. } => GoalState::Stroll(Default::default()),
        GoalKind::LookAtPlayer { .. } => GoalState::LookAtPlayer(Default::default()),
        GoalKind::LookAround => GoalState::LookAround(Default::default()),
        GoalKind::Melee { .. } => GoalState::Melee(Default::default()),
        GoalKind::Leap { .. } => GoalState::Leap(Default::default()),
        GoalKind::Avoid { .. } => GoalState::Avoid(Default::default()),
        GoalKind::Ranged { .. } => GoalState::Ranged(Default::default()),
        GoalKind::FollowOwner { .. } => GoalState::FollowOwner(Default::default()),
        GoalKind::Beg { .. } => GoalState::Beg(Default::default()),
        GoalKind::NearestTarget { .. } => GoalState::Target(Default::default()),
        GoalKind::HurtBy { .. } => GoalState::HurtBy(Default::default()),
        GoalKind::OwnerHurtBy | GoalKind::OwnerHurt => GoalState::Owner(Default::default()),
        GoalKind::ResetAnger { .. } => GoalState::Anger(Default::default()),
        GoalKind::Float
        | GoalKind::Swell
        | GoalKind::FleeSun { .. }
        | GoalKind::Panic { .. }
        | GoalKind::Sit => GoalState::None,
    }
}

impl<W: GoalWorld + ?Sized> MobBrain<W> for TableBrain {
    fn rules(&self) -> &'static BrainRules {
        &self.table.rules
    }

    fn evaluate(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        intents: &mut Intents,
    ) -> u64 {
        let view = turn.view;
        let mut target = view.target;
        let mut wake = Wake::default();
        if view.in_love() || view.is_baby() || view.target.is_some() {
            wake.busy = true;
        }
        if !self.panics() && panic::panic_record_fresh(view, turn.next_evaluation()) {
            wake.busy = true;
        }
        let disabled = view.flags.disabled();
        let split = self.table.targets.len().min(self.state.len());
        {
            let mut cx = Cx {
                turn,
                scratch,
                intents,
                path: &mut self.path,
                target: &mut target,
                wake: &mut wake,
                destination: None,
            };
            let (target_state, goal_state) = self.state.split_at_mut(split);
            let mut run = ListRun {
                cx: &mut cx,
                rows: self.table.targets,
                state: target_state,
                examined: 0,
            };
            self.targets.evaluate(&mut run, disabled);
            run.end_unexamined_gates();
            let mut run = ListRun {
                cx: &mut cx,
                rows: self.table.goals,
                state: goal_state,
                examined: 0,
            };
            self.goals.evaluate(&mut run, disabled);
            run.end_unexamined_gates();
        }
        let panic = if self.panics() {
            Panic::Panicking
        } else {
            Panic::Calm
        };
        if panic != self.published {
            self.published = panic;
            intents.panic = Some(panic);
        }
        let next = turn.next_evaluation();
        if wake.changed || wake.busy {
            return next;
        }
        wake.due
            .unwrap_or(u64::MAX)
            .max(turn.tick.saturating_add(1))
    }
}

struct ListRun<'x, 'r, 't, 'a, W: ?Sized> {
    cx: &'x mut Cx<'r, 't, 'a, W>,
    rows: &'static [GoalRow],
    state: &'x mut [GoalState],
    examined: u32,
}

impl<W: GoalWorld + ?Sized> ListRun<'_, '_, '_, '_, W> {
    fn with_state(
        &mut self,
        goal: usize,
        act: impl FnOnce(&mut Cx<'_, '_, '_, W>, &GoalRow, &mut GoalState) -> bool,
    ) -> bool {
        let Some(row) = self.rows.get(goal) else {
            return false;
        };
        let Some(slot) = self.state.get_mut(goal) else {
            return false;
        };
        let mut state = *slot;
        let result = act(self.cx, row, &mut state);
        *slot = state;
        result
    }

    fn end_unexamined_gates(&mut self) {
        for (goal, slot) in self.state.iter_mut().enumerate() {
            if goal < 32 && self.examined & (1 << goal) != 0 {
                continue;
            }
            match slot {
                GoalState::Stroll(stroll) => stroll.gate_mut().end(),
                GoalState::LookAtPlayer(look) => look.gate_mut().end(),
                GoalState::LookAround(look) => look.gate_mut().end(),
                GoalState::Eat(eat) => eat.gate.end(),
                GoalState::Leap(gate) => gate.end(),
                GoalState::Target(target) => target.gate.end(),
                _ => {}
            }
        }
    }
}

fn nearest_params(goal: &GoalKind) -> Option<NearestParams> {
    match *goal {
        GoalKind::NearestTarget {
            set,
            interval,
            sight,
            filter,
            follow_scale,
            tame,
        } => Some(NearestParams {
            set,
            interval,
            sight,
            filter,
            follow_scale,
            tame,
        }),
        _ => None,
    }
}

fn avoid_params(goal: &GoalKind) -> Option<AvoidParams> {
    match *goal {
        GoalKind::Avoid {
            set,
            distance,
            walk,
            sprint,
            extra,
        } => Some(AvoidParams {
            set,
            distance,
            walk,
            sprint,
            extra,
        }),
        _ => None,
    }
}

fn ranged_params(goal: &GoalKind) -> Option<RangedParams> {
    match *goal {
        GoalKind::Ranged {
            speed,
            min_interval,
            max_interval,
            range,
        } => Some(RangedParams {
            speed,
            min_interval,
            max_interval,
            range,
        }),
        _ => None,
    }
}

fn note_gate<W: GoalWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, due: Option<u64>) {
    match due {
        Some(due) => cx.wake.due_at(Some(due)),
        None => cx.wake.busy = true,
    }
}

impl<W: GoalWorld + ?Sized> Goals for ListRun<'_, '_, '_, '_, W> {
    fn count(&self) -> usize {
        self.rows.len()
    }

    fn spec(&self, goal: usize) -> Option<GoalSpec> {
        self.rows.get(goal).map(GoalRow::spec)
    }

    fn can_start(&mut self, goal: usize) -> bool {
        if goal < 32 {
            self.examined |= 1 << goal;
        }
        self.with_state(goal, |cx, row, state| match (&row.goal, state) {
            (GoalKind::Float, _) => cx.me().flags.contains(MobFlags::AFLOAT),
            (GoalKind::Swell, _) => cx.me().flags.contains(MobFlags::SWELLING),
            (GoalKind::Panic { .. }, _) => {
                if !panic::panic_record_fresh(cx.me(), cx.tick()) {
                    return false;
                }
                cx.busy();
                cx.destination = panic::panic_destination(cx.turn);
                cx.destination.is_some()
            }
            (GoalKind::FleeSun { .. }, _) => {
                let me = *cx.me();
                if cx.target.is_some()
                    || !me.flags.contains(MobFlags::ON_FIRE)
                    || !me.flags.contains(MobFlags::IN_SUN)
                {
                    return false;
                }
                cx.busy();
                cx.destination = crate::random_position::default_search(
                    cx.turn.world,
                    me.position,
                    crate::SearchRange::new(20, 3),
                    &mut cx.turn.random(crate::FLEE_SUN_SEARCH),
                );
                cx.destination.is_some()
            }
            (GoalKind::Breed { kinds, mates, .. }, GoalState::Breed(state)) => {
                breed::can_start(cx, *kinds, *mates, state)
            }
            (GoalKind::Tempt { items, .. }, GoalState::Tempt(state)) => {
                tempt::can_start(cx, *items, state)
            }
            (GoalKind::FollowAdult { kinds, .. }, GoalState::Follow(state)) => {
                follow_adult::can_start(cx, *kinds, state)
            }
            (GoalKind::EatGrass, GoalState::Eat(state)) => eat_grass::can_start(cx, state),
            (GoalKind::Stroll { .. }, GoalState::Stroll(stroll)) => {
                cx.destination = stroll.start_destination(cx.turn);
                note_gate(cx, stroll.gate().due());
                cx.destination.is_some()
            }
            (GoalKind::LookAtPlayer { range }, GoalState::LookAtPlayer(look)) => {
                let started = look.can_start(cx.turn, f64::from(*range));
                note_gate(cx, look.gate().due());
                started
            }
            (GoalKind::LookAround, GoalState::LookAround(look)) => {
                let started = look.can_start(cx.turn);
                note_gate(cx, look.gate().due());
                started
            }
            (GoalKind::Melee { .. }, GoalState::Melee(state)) => melee::can_start(cx, state),
            (GoalKind::Leap { .. }, GoalState::Leap(gate)) => {
                let started = leap::can_start(cx, gate);
                if !started && gate.due().is_some() {
                    cx.wake.due_at(gate.due());
                }
                started
            }
            (GoalKind::Avoid { .. }, GoalState::Avoid(state)) => {
                avoid_params(&row.goal).is_some_and(|params| avoid::can_start(cx, &params, state))
            }
            (GoalKind::Ranged { .. }, GoalState::Ranged(state)) => ranged::can_start(cx, state),
            (GoalKind::FollowOwner { start, .. }, _) => owner::follow_can_start(cx, *start),
            (GoalKind::Sit, _) => owner::sit_can_start(cx),
            (GoalKind::Beg { range, items }, GoalState::Beg(state)) => {
                owner::beg_can_start(cx, *range, *items, state)
            }
            (GoalKind::NearestTarget { .. }, GoalState::Target(state)) => nearest_params(&row.goal)
                .is_some_and(|params| nearest_target::can_start(cx, &params, goal, state)),
            (GoalKind::HurtBy { ignored, .. }, GoalState::HurtBy(state)) => {
                hurt_by::can_start(cx, ignored, state)
            }
            (GoalKind::OwnerHurtBy, GoalState::Owner(state)) => {
                owner::owner_hurt_by_can_start(cx, state)
            }
            (GoalKind::OwnerHurt | GoalKind::ResetAnger { .. }, _) => false,
            _ => false,
        })
    }

    fn can_continue(&mut self, goal: usize) -> bool {
        self.with_state(goal, |cx, row, state| match (&row.goal, state) {
            (GoalKind::Float, _) => cx.me().flags.contains(MobFlags::AFLOAT),
            (GoalKind::Swell, _) => cx.me().flags.contains(MobFlags::SWELLING),
            (GoalKind::Panic { .. } | GoalKind::FleeSun { .. }, _) => cx.path_running(),
            (GoalKind::Breed { .. }, GoalState::Breed(state)) => breed::can_continue(cx, state),
            (GoalKind::Tempt { items, .. }, GoalState::Tempt(state)) => {
                tempt::can_start(cx, *items, state)
            }
            (GoalKind::FollowAdult { .. }, GoalState::Follow(state)) => {
                follow_adult::can_continue(cx, state)
            }
            (GoalKind::EatGrass, GoalState::Eat(state)) => eat_grass::can_continue(cx, state),
            (GoalKind::Stroll { .. }, _) => {
                let running = cx.path_running();
                crate::StrollGoal::can_continue(cx.turn, running)
            }
            (GoalKind::LookAtPlayer { range }, GoalState::LookAtPlayer(look)) => {
                cx.busy();
                look.can_continue(cx.turn, f64::from(*range))
            }
            (GoalKind::LookAround, GoalState::LookAround(look)) => {
                cx.wake.due_at(look.ends());
                look.can_continue(cx.turn)
            }
            (GoalKind::Melee { pursuit, .. }, _) => melee::can_continue(cx, *pursuit),
            (GoalKind::Leap { .. }, _) => leap::can_continue(cx),
            (GoalKind::Avoid { .. }, _) => avoid::can_continue(cx),
            (GoalKind::Ranged { .. }, GoalState::Ranged(state)) => ranged::can_continue(cx, state),
            (GoalKind::FollowOwner { stop, .. }, _) => owner::follow_can_continue(cx, *stop),
            (GoalKind::Sit, _) => owner::sit_can_continue(cx),
            (GoalKind::Beg { range, items }, GoalState::Beg(state)) => {
                owner::beg_can_continue(cx, *range, *items, state)
            }
            (GoalKind::NearestTarget { .. }, GoalState::Target(state)) => nearest_params(&row.goal)
                .is_some_and(|params| nearest_target::can_continue(cx, &params, state)),
            (GoalKind::HurtBy { .. }, GoalState::HurtBy(state)) => hurt_by::can_continue(cx, state),
            (GoalKind::OwnerHurtBy | GoalKind::OwnerHurt, GoalState::Owner(state)) => {
                owner::owner_can_continue(cx, state)
            }
            (GoalKind::ResetAnger { .. }, _) => false,
            _ => false,
        })
    }

    fn start(&mut self, goal: usize) {
        self.cx.wake.changed = true;
        self.with_state(goal, |cx, row, state| {
            match (&row.goal, state) {
                (GoalKind::Panic { speed } | GoalKind::FleeSun { speed }, _) => {
                    if let Some(destination) = cx.destination.take() {
                        cx.request_path(destination, *speed);
                    }
                }
                (GoalKind::Stroll { speed, .. }, _) => {
                    if let Some(destination) = cx.destination.take() {
                        cx.request_path(destination, *speed);
                    }
                }
                (GoalKind::FollowAdult { .. }, GoalState::Follow(state)) => {
                    follow_adult::start(state);
                }
                (GoalKind::EatGrass, GoalState::Eat(state)) => eat_grass::start(cx, state),
                (GoalKind::LookAtPlayer { .. }, GoalState::LookAtPlayer(look)) => {
                    look.start(cx.turn);
                }
                (GoalKind::LookAround, GoalState::LookAround(look)) => {
                    look.start(cx.turn, cx.intents);
                }
                (
                    GoalKind::Melee {
                        speed,
                        pursuit,
                        interval,
                    },
                    GoalState::Melee(state),
                ) => melee::start(cx, *speed, *pursuit, *interval, state),
                (GoalKind::Leap { vertical }, _) => leap::start(cx, *vertical),
                (GoalKind::Avoid { .. }, GoalState::Avoid(state)) => {
                    if let Some(params) = avoid_params(&row.goal) {
                        avoid::start(cx, &params, state);
                    }
                }
                (GoalKind::Ranged { .. }, _) => {
                    if let Some(params) = ranged_params(&row.goal) {
                        ranged::start(cx, &params);
                    }
                }
                (GoalKind::FollowOwner { .. }, GoalState::FollowOwner(state)) => {
                    owner::follow_start(state);
                }
                (GoalKind::Sit, _) => owner::sit_start(cx),
                (GoalKind::Beg { .. }, GoalState::Beg(state)) => owner::beg_start(cx, state),
                (GoalKind::NearestTarget { .. }, GoalState::Target(state)) => {
                    nearest_target::start(cx, state);
                }
                (GoalKind::HurtBy { alert, .. }, GoalState::HurtBy(state)) => {
                    hurt_by::start(cx, *alert, state);
                }
                (GoalKind::OwnerHurtBy, GoalState::Owner(state)) => {
                    owner::owner_hurt_by_start(cx, state);
                }
                _ => {}
            }
            true
        });
    }

    fn stop(&mut self, goal: usize) {
        self.cx.wake.changed = true;
        self.with_state(goal, |cx, row, state| {
            match (&row.goal, state) {
                (GoalKind::Breed { .. }, GoalState::Breed(state)) => breed::stop(state),
                (GoalKind::Tempt { .. }, GoalState::Tempt(state)) => tempt::stop(cx, state),
                (GoalKind::FollowAdult { .. }, GoalState::Follow(state)) => {
                    follow_adult::stop(state);
                }
                (GoalKind::EatGrass, GoalState::Eat(state)) => eat_grass::stop(cx, state),
                (GoalKind::Stroll { .. }, _) => cx.clear_path(),
                (GoalKind::LookAtPlayer { .. }, GoalState::LookAtPlayer(look)) => look.stop(),
                (GoalKind::LookAround, GoalState::LookAround(look)) => look.stop(cx.intents),
                (GoalKind::Melee { .. }, _) => melee::stop(cx),
                (GoalKind::Avoid { .. }, GoalState::Avoid(state)) => avoid::stop(state),
                (GoalKind::Ranged { .. }, GoalState::Ranged(state)) => ranged::stop(cx, state),
                (GoalKind::FollowOwner { .. }, _) => owner::follow_stop(cx),
                (GoalKind::Sit, _) => owner::sit_stop(cx),
                (GoalKind::Beg { .. }, GoalState::Beg(state)) => owner::beg_stop(cx, state),
                (GoalKind::NearestTarget { .. }, GoalState::Target(state)) => {
                    nearest_target::stop(cx, state);
                }
                (GoalKind::HurtBy { .. }, GoalState::HurtBy(state)) => hurt_by::stop(cx, state),
                (GoalKind::OwnerHurtBy | GoalKind::OwnerHurt, _) => owner::owner_stop(cx),
                _ => {}
            }
            true
        });
    }

    fn tick(&mut self, goal: usize) {
        self.with_state(goal, |cx, row, state| {
            match (&row.goal, state) {
                (GoalKind::Breed { speed, .. }, GoalState::Breed(state)) => {
                    breed::tick(cx, *speed, state);
                }
                (GoalKind::Tempt { speed, .. }, GoalState::Tempt(state)) => {
                    tempt::tick(cx, *speed, state);
                }
                (GoalKind::FollowAdult { speed, .. }, GoalState::Follow(state)) => {
                    follow_adult::tick(cx, *speed, state);
                }
                (GoalKind::EatGrass, GoalState::Eat(state)) => eat_grass::tick(state),
                (GoalKind::LookAtPlayer { .. }, GoalState::LookAtPlayer(look)) => {
                    look.tick(cx.turn, cx.intents);
                }
                (GoalKind::Melee { speed, pursuit, .. }, GoalState::Melee(state)) => {
                    melee::tick(cx, *speed, *pursuit, state);
                }
                (GoalKind::Avoid { .. }, GoalState::Avoid(state)) => {
                    if let Some(params) = avoid_params(&row.goal) {
                        avoid::tick(cx, &params, state);
                    }
                }
                (GoalKind::Ranged { .. }, GoalState::Ranged(state)) => {
                    if let Some(params) = ranged_params(&row.goal) {
                        ranged::tick(cx, &params, state);
                    }
                }
                (GoalKind::FollowOwner { speed, .. }, GoalState::FollowOwner(state)) => {
                    owner::follow_tick(cx, *speed, state);
                }
                (GoalKind::Beg { .. }, GoalState::Beg(state)) => owner::beg_tick(cx, state),
                _ => {}
            }
            true
        });
    }
}
