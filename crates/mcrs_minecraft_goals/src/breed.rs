use mcrs_minecraft_brain::Candidate;
use mcrs_minecraft_brain::CandidateFlags;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::Proposal;
use mcrs_minecraft_goal_world::GoalWorld;

use crate::BreedState;
use crate::Kinds;
use crate::Mates;
use crate::query::Cx;

const PARTNER_RANGE: f64 = 8.0;
const APPROACH_UNITS: u8 = 30;
const GIVE_UP_UNITS: u8 = 60;
const BREED_DISTANCE_SQUARED: f64 = 9.0;
const LOOK_SPEED: f32 = 10.0;

pub(crate) fn kind_set(kinds: Kinds, own: mcrs_minecraft_brain::MobKind) -> KindSet {
    match kinds {
        Kinds::Own => KindSet::NONE.with(own),
        Kinds::Set(set) => set,
    }
}

fn can_mate<W: GoalWorld + ?Sized>(
    cx: &Cx<'_, '_, '_, W>,
    candidate: &Candidate,
    mates: Mates,
) -> bool {
    let me = cx.me();
    match mates {
        Mates::InLove => candidate.in_love(),
        Mates::TameInLove => {
            me.flags.contains(MobFlags::TAME)
                && candidate.flags.contains(CandidateFlags::TAME)
                && !candidate.flags.contains(CandidateFlags::SITTING)
                && candidate.in_love()
        }
        Mates::TameAdultsAtFullHealth => {
            me.flags.contains(MobFlags::TAME)
                && !me.is_baby()
                && !me.flags.contains(MobFlags::PASSENGER)
                && !me.flags.contains(MobFlags::HAS_PASSENGER)
                && candidate.flags.contains(CandidateFlags::TAME)
                && candidate.in_love()
                && !candidate.is_baby()
                && !candidate.flags.contains(CandidateFlags::PASSENGER)
                && !candidate.flags.contains(CandidateFlags::HAS_PASSENGER)
        }
    }
}

pub(crate) fn can_start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    kinds: Kinds,
    mates: Mates,
    state: &mut BreedState,
) -> bool {
    let me = *cx.me();
    if !me.in_love() {
        return false;
    }
    let set = kind_set(kinds, me.kind);
    let (min, max) = cx.grown_box(PARTNER_RANGE, PARTNER_RANGE);
    cx.collect_mobs(min, max, &set);
    let partner = cx.nearest(
        |cx, candidate| cx.feet_distance_squared(candidate),
        |cx, candidate| {
            let reach = (PARTNER_RANGE * f64::from(candidate.visibility)).max(2.0);
            candidate.is_alive()
                && !candidate.flags.contains(CandidateFlags::SPECTATOR)
                && !candidate.flags.contains(CandidateFlags::IN_PANIC)
                && can_mate(cx, candidate, mates)
                && cx.feet_distance_squared(candidate) <= reach * reach
        },
    );
    state.partner = EntityRef::some(partner.map(|candidate| candidate.id));
    state.count = 0;
    partner.is_some()
}

pub(crate) fn tick<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    speed: f32,
    state: &mut BreedState,
) {
    let Some(partner) = state.partner.get().and_then(|id| cx.find(id)) else {
        return;
    };
    let pitch = cx.pitch_limit();
    cx.look_at(partner.eye(), Some(LOOK_SPEED), Some(pitch));
    cx.request_path_to(&partner, speed);
    state.count = state.count.saturating_add(1);
    if state.count >= APPROACH_UNITS && cx.feet_distance_squared(&partner) < BREED_DISTANCE_SQUARED
    {
        cx.intents.proposal = Some(Proposal::Breed {
            partner: partner.id,
        });
    }
}

pub(crate) fn can_continue<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &BreedState,
) -> bool {
    cx.busy();
    let Some(partner) = state.partner.get().and_then(|id| cx.find(id)) else {
        return false;
    };
    partner.is_alive()
        && partner.in_love()
        && !partner.flags.contains(CandidateFlags::IN_PANIC)
        && state.count < GIVE_UP_UNITS
}

pub(crate) fn stop(state: &mut BreedState) {
    state.partner = EntityRef::NONE;
    state.count = 0;
}
