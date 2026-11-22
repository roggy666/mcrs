use mcrs_minecraft_brain::Candidate;
use mcrs_minecraft_brain::CandidateFlags;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::MobFlags;

use crate::BrainWorld;
use crate::Sight;
use crate::TARGET_WAIT;
use crate::TameRule;
use crate::TargetFilter;
use crate::TargetSet;
use crate::TargetState;
use crate::query::Combat;
use crate::query::Cx;
use crate::query::Ray;
use crate::query::Visibility;

pub(crate) const UNSEEN_LIMIT: u8 = 30;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NearestParams {
    pub(crate) set: TargetSet,
    pub(crate) interval: u8,
    pub(crate) sight: Sight,
    pub(crate) filter: TargetFilter,
    pub(crate) follow_scale: f32,
    pub(crate) tame: TameRule,
}

fn passes_filter<W: BrainWorld + ?Sized>(
    cx: &Cx<'_, '_, '_, W>,
    candidate: &Candidate,
    filter: TargetFilter,
) -> bool {
    match filter {
        TargetFilter::None => true,
        TargetFilter::BabyOnLand => {
            candidate.is_baby() && !candidate.flags.contains(CandidateFlags::IN_WATER)
        }
        TargetFilter::AngryAt => {
            cx.me().flags.contains(MobFlags::ANGRY) && cx.target.is(candidate.id)
        }
        TargetFilter::NoOwner => candidate.owner.is_none(),
    }
}

pub(crate) fn can_start<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    params: &NearestParams,
    slot: usize,
    state: &mut TargetState,
) -> bool {
    if params.tame == TameRule::NotTame && cx.me().flags.contains(MobFlags::TAME) {
        state.gate.end();
        return false;
    }
    let probe = match params.set {
        TargetSet::Players => cx.turn.players.iter().any(|row| row.is_attackable()),
        TargetSet::Kinds(kinds) => cx.turn.neighbours.has_kind(&kinds),
    };
    if !probe {
        state.gate.end();
        return false;
    }
    if params.interval > 0 {
        let units = f64::from(params.interval.div_ceil(2));
        let part = crate::slot_part(TARGET_WAIT, slot);
        if !state.gate.passes(cx.turn, part, 1.0 / units) {
            cx.wake.due_at(state.gate.due());
            return false;
        }
    }
    let follow = cx.follow_range() * f64::from(params.follow_scale);
    let combat = Combat {
        range: follow,
        ray: Ray::Check,
        visibility: Visibility::Use,
    };
    match params.set {
        TargetSet::Players => cx.collect_players(),
        TargetSet::Kinds(kinds) => {
            let (min, max) = cx.grown_box(follow, follow);
            cx.collect_mobs(min, max, &kinds);
        }
    }
    let filter = params.filter;
    let found = cx.nearest_sorted(
        |cx, candidate| cx.select_distance(candidate),
        |cx, candidate| passes_filter(cx, candidate, filter) && cx.combat_test(candidate, combat),
    );
    cx.wake.busy = true;
    match found {
        Some(candidate) => {
            state.kept = EntityRef::of(candidate.id);
            true
        }
        None => false,
    }
}

pub(crate) fn start<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut TargetState) {
    cx.set_target(state.kept.get());
    state.unseen = 0;
}

pub(crate) fn can_continue<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    params: &NearestParams,
    state: &mut TargetState,
) -> bool {
    cx.busy();
    let follow = cx.follow_range() * f64::from(params.follow_scale);
    match params.tame {
        TameRule::NotTame => {
            let Some(kept) = state.kept.get().and_then(|id| cx.find(id)) else {
                return false;
            };
            cx.combat_test(
                &kept,
                Combat {
                    range: follow,
                    ray: Ray::Check,
                    visibility: Visibility::Use,
                },
            )
        }
        TameRule::Any => base_continue(
            cx,
            params.sight,
            follow,
            &mut state.unseen,
            UNSEEN_LIMIT,
            EntityRef::NONE,
        ),
    }
}

pub(crate) fn base_continue<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    sight: Sight,
    follow: f64,
    unseen: &mut u8,
    limit: u8,
    goal_target: EntityRef,
) -> bool {
    let Some(id) = cx.target.get().or(goal_target.get()) else {
        return false;
    };
    let Some(candidate) = cx.find(id) else {
        return false;
    };
    if !cx.can_attack(&candidate) {
        return false;
    }
    if cx.feet_distance_squared(&candidate) > follow * follow {
        return false;
    }
    if sight == Sight::MustSee {
        if cx.sees(&candidate) {
            *unseen = 0;
        } else {
            *unseen = unseen.saturating_add(1);
            if *unseen > limit {
                return false;
            }
        }
    }
    cx.set_target(Some(candidate.id));
    true
}

pub(crate) fn stop<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut TargetState) {
    cx.set_target(None);
    state.kept = EntityRef::NONE;
}
