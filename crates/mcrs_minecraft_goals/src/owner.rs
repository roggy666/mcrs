use glam::DVec3;
use mcrs_minecraft_brain::Candidate;
use mcrs_minecraft_brain::CandidateFlags;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::ItemTest;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::PoseBits;
use mcrs_minecraft_brain::PoseIntent;

use crate::BEG_TIME;
use crate::BegState;
use crate::BrainWorld;
use crate::FollowOwnerState;
use crate::OwnerState;
use crate::Sight;
use crate::nearest_target::UNSEEN_LIMIT;
use crate::nearest_target::base_continue;
use crate::query::Combat;
use crate::query::Cx;
use crate::query::Ray;
use crate::query::Visibility;

const FAR_SQUARED: f64 = 144.0;
const OWNER_LOOK_SPEED: f32 = 10.0;
const PATH_UNITS: u8 = 5;
const OWNER_HURT_WINDOW: u32 = 100;

pub(crate) fn owner<W: BrainWorld + ?Sized>(cx: &Cx<'_, '_, '_, W>) -> Option<Candidate> {
    cx.me().owner.get().and_then(|id| cx.find(id))
}

fn can_move_to_owner<W: BrainWorld + ?Sized>(cx: &Cx<'_, '_, '_, W>, owner: &Candidate) -> bool {
    let me = cx.me();
    !me.flags.contains(MobFlags::SITTING)
        && !me.flags.contains(MobFlags::PASSENGER)
        && !me.flags.contains(MobFlags::LEASHED)
        && !owner.flags.contains(CandidateFlags::SPECTATOR)
}

pub(crate) fn follow_can_start<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    start: f32,
) -> bool {
    let Some(owner) = owner(cx) else {
        return false;
    };
    cx.busy();
    let start = f64::from(start);
    can_move_to_owner(cx, &owner) && cx.feet_distance_squared(&owner) >= start * start
}

pub(crate) fn follow_start(state: &mut FollowOwnerState) {
    state.timer = 0;
}

pub(crate) fn follow_tick<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    speed: f32,
    state: &mut FollowOwnerState,
) {
    let Some(owner) = owner(cx) else {
        return;
    };
    let far = cx.feet_distance_squared(&owner) >= FAR_SQUARED;
    if !far {
        let pitch = cx.pitch_limit();
        cx.look_at(owner.eye(), Some(OWNER_LOOK_SPEED), Some(pitch));
    }
    state.timer = state.timer.saturating_sub(1);
    if state.timer > 0 {
        return;
    }
    state.timer = PATH_UNITS;
    cx.request_path_to(&owner, speed);
}

pub(crate) fn follow_can_continue<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    stop: f32,
) -> bool {
    cx.busy();
    let Some(owner) = owner(cx) else {
        return false;
    };
    let stop = f64::from(stop);
    cx.path_running()
        && can_move_to_owner(cx, &owner)
        && cx.feet_distance_squared(&owner) > stop * stop
}

pub(crate) fn follow_stop<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) {
    cx.clear_path();
}

pub(crate) fn sit_can_start<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) -> bool {
    let me = *cx.me();
    if !me.flags.contains(MobFlags::SITTING) && !me.flags.contains(MobFlags::TAME) {
        return false;
    }
    if me.flags.contains(MobFlags::IN_WATER) || !me.flags.contains(MobFlags::ON_GROUND) {
        return false;
    }
    let Some(owner) = owner(cx) else {
        return true;
    };
    if cx.feet_distance_squared(&owner) < FAR_SQUARED && !owner.hurt_by.is_none() {
        return false;
    }
    me.flags.contains(MobFlags::SITTING)
}

pub(crate) fn sit_start<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) {
    cx.clear_path();
    cx.intents.pose = Some(
        cx.intents
            .pose
            .unwrap_or_default()
            .merged(PoseIntent::set(PoseBits::SITTING)),
    );
}

pub(crate) fn sit_can_continue<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) -> bool {
    cx.me().flags.contains(MobFlags::SITTING)
}

pub(crate) fn sit_stop<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) {
    cx.intents.pose = Some(
        cx.intents
            .pose
            .unwrap_or_default()
            .merged(PoseIntent::clear(PoseBits::SITTING)),
    );
}

pub(crate) fn beg_can_start<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    range: f32,
    items: ItemTest,
    state: &mut BegState,
) -> bool {
    let range = f64::from(range);
    let tables = cx.turn.items;
    let interested = cx.turn.players.iter().any(|row| {
        let reach = (range * f64::from(row.visibility)).max(2.0);
        row.is_qualified()
            && row.holds(|item| items.passes(tables, item))
            && row.position.distance_squared(cx.me().position) <= reach * reach
    });
    if !interested {
        return false;
    }
    cx.busy();
    cx.collect_players();
    let combat = Combat {
        range,
        ray: Ray::Check,
        visibility: Visibility::Use,
    };
    let Some(player) = cx.nearest_sorted(
        |cx, candidate| cx.feet_distance_squared(candidate),
        |cx, candidate| {
            candidate.is_alive()
                && !candidate.flags.contains(CandidateFlags::SPECTATOR)
                && cx.in_range_and_sight(candidate, combat)
        },
    ) else {
        return false;
    };
    if !player.hands.iter().any(|item| items.passes(tables, *item)) {
        return false;
    }
    state.player = EntityRef::of(player.id);
    true
}

pub(crate) fn beg_start<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut BegState) {
    cx.intents.pose = Some(
        cx.intents
            .pose
            .unwrap_or_default()
            .merged(PoseIntent::set(PoseBits::INTERESTED)),
    );
    let draw = cx.turn.random(BEG_TIME).below(40);
    state.time = u8::try_from((40 + draw).div_ceil(2)).unwrap_or(u8::MAX);
}

pub(crate) fn beg_tick<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut BegState) {
    if let Some(player) = state.player.get().and_then(|id| cx.find(id)) {
        let eye = cx.eye();
        let point = DVec3::new(player.position.x, eye.y, player.position.z);
        let pitch = cx.pitch_limit();
        cx.look_at(point, Some(OWNER_LOOK_SPEED), Some(pitch));
    }
    state.time = state.time.saturating_sub(1);
}

pub(crate) fn beg_can_continue<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    range: f32,
    items: ItemTest,
    state: &BegState,
) -> bool {
    cx.busy();
    let Some(player) = state.player.get().and_then(|id| cx.find(id)) else {
        return false;
    };
    let range = f64::from(range);
    let tables = cx.turn.items;
    player.is_alive()
        && cx.feet_distance_squared(&player) <= range * range
        && state.time > 0
        && player.hands.iter().any(|item| items.passes(tables, *item))
}

pub(crate) fn beg_stop<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut BegState) {
    cx.intents.pose = Some(
        cx.intents
            .pose
            .unwrap_or_default()
            .merged(PoseIntent::clear(PoseBits::INTERESTED)),
    );
    state.player = EntityRef::NONE;
}

fn wants_to_attack<W: BrainWorld + ?Sized>(
    cx: &Cx<'_, '_, '_, W>,
    candidate: &Candidate,
    owner: &Candidate,
) -> bool {
    if candidate.flags.contains(CandidateFlags::TAME) && candidate.owner.is(owner.id) {
        return false;
    }
    if candidate.flags.contains(CandidateFlags::TAME) {
        return false;
    }
    cx.can_attack(candidate)
}

pub(crate) fn owner_hurt_by_can_start<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &OwnerState,
) -> bool {
    let me = *cx.me();
    if !me.flags.contains(MobFlags::TAME) || me.flags.contains(MobFlags::SITTING) {
        return false;
    }
    let Some(owner) = owner(cx) else {
        return false;
    };
    cx.busy();
    let Some((attacker, tick)) = owner.hurt_by.get() else {
        return false;
    };
    if (cx.tick() as u32).wrapping_sub(tick) > OWNER_HURT_WINDOW || tick == state.kept_tick {
        return false;
    }
    let Some(candidate) = cx.find(attacker) else {
        return false;
    };
    cx.combat_test(
        &candidate,
        Combat {
            range: 0.0,
            ray: Ray::Check,
            visibility: Visibility::Use,
        },
    ) && wants_to_attack(cx, &candidate, &owner)
}

pub(crate) fn owner_hurt_by_start<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &mut OwnerState,
) {
    let Some(owner) = owner(cx) else {
        return;
    };
    let Some((attacker, tick)) = owner.hurt_by.get() else {
        return;
    };
    cx.set_target(Some(attacker));
    state.kept_tick = tick;
    state.unseen = 0;
}

pub(crate) fn owner_can_continue<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &mut OwnerState,
) -> bool {
    cx.busy();
    let follow = cx.follow_range();
    base_continue(
        cx,
        Sight::Any,
        follow,
        &mut state.unseen,
        UNSEEN_LIMIT,
        EntityRef::NONE,
    )
}

pub(crate) fn owner_stop<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) {
    cx.set_target(None);
}
