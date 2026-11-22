use mcrs_minecraft_brain::Candidate;
use mcrs_minecraft_brain::CandidateFlags;
use mcrs_minecraft_brain::MeleeIntent;
use mcrs_minecraft_brain::PoseBits;
use mcrs_minecraft_brain::PoseIntent;
use mcrs_minecraft_brain::Pursuit;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_voxel::BlockPos;

use crate::MELEE_PATH;
use crate::MeleeState;
use crate::query::Cx;

const CHECK_TICKS: u32 = 20;
const REACH_GROWTH: f64 = 0.828_427_1 - 0.6;
const REPATH_CHANCE: f64 = 0.05;
const FAR_SQUARED: f64 = 1024.0;
const MID_SQUARED: f64 = 256.0;

#[must_use]
pub fn in_reach(my_min: glam::DVec3, my_max: glam::DVec3, candidate: &Candidate) -> bool {
    let min = glam::DVec3::new(my_min.x - REACH_GROWTH, my_min.y, my_min.z - REACH_GROWTH);
    let max = glam::DVec3::new(my_max.x + REACH_GROWTH, my_max.y, my_max.z + REACH_GROWTH);
    candidate.intersects(min, max)
}

fn target_in_reach<W: GoalWorld + ?Sized>(cx: &Cx<'_, '_, '_, W>, candidate: &Candidate) -> bool {
    let (min, max) = cx.grown_box(0.0, 0.0);
    in_reach(min, max, candidate)
}

pub(crate) fn can_start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &mut MeleeState,
) -> bool {
    let now = cx.tick() as u32;
    if now.wrapping_sub(state.last_check) < CHECK_TICKS && state.last_check != 0 {
        if cx.target.is_some() {
            cx.busy();
        }
        return false;
    }
    state.last_check = now.max(1);
    let Some(target) = cx.attack_target() else {
        return false;
    };
    cx.busy();
    cx.has_route_to(&target) || target_in_reach(cx, &target)
}

pub(crate) fn start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    speed: f32,
    pursuit: Pursuit,
    interval: u8,
    state: &mut MeleeState,
) {
    if let Some(target) = cx.attack_target() {
        cx.request_path_to(&target, speed);
        state.point = Some(BlockPos::from(target.position));
    }
    cx.intents.melee = Some(MeleeIntent::Start {
        speed_modifier: speed,
        interval,
        pursuit,
    });
    cx.intents.pose = Some(
        cx.intents
            .pose
            .unwrap_or_default()
            .merged(PoseIntent::set(PoseBits::AGGRESSIVE)),
    );
    state.timer = 0;
}

pub(crate) fn can_continue<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    pursuit: Pursuit,
) -> bool {
    cx.busy();
    let Some(target) = cx.attack_target() else {
        return false;
    };
    match pursuit {
        Pursuit::Unseen => {
            !(target.is_player()
                && (target.flags.contains(CandidateFlags::CREATIVE)
                    || target.flags.contains(CandidateFlags::SPECTATOR)))
        }
        Pursuit::Seen => cx.path_running(),
    }
}

pub(crate) fn tick<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    speed: f32,
    pursuit: Pursuit,
    state: &mut MeleeState,
) {
    let Some(target) = cx.attack_target() else {
        return;
    };
    state.timer = state.timer.saturating_sub(2);
    let sees = pursuit == Pursuit::Unseen || cx.sees(&target);
    if !sees || state.timer > 0 {
        return;
    }
    let feet = BlockPos::from(target.position);
    let moved = state
        .point
        .is_none_or(|point| point.bottom_centre().distance_squared(target.position) >= 1.0);
    let repath = moved || cx.turn.random(MELEE_PATH).chance(REPATH_CHANCE);
    if !repath {
        return;
    }
    state.point = Some(feet);
    let mut timer = 4 + cx.turn.random(MELEE_PATH).below(7) as u8;
    let distance = cx.feet_distance_squared(&target);
    if distance > FAR_SQUARED {
        timer = timer.saturating_add(10);
    } else if distance > MID_SQUARED {
        timer = timer.saturating_add(5);
    }
    if !cx.request_path_to(&target, speed) {
        timer = timer.saturating_add(15);
    }
    state.timer = timer;
}

pub(crate) fn stop<W: GoalWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) {
    if let Some(id) = cx.target.get()
        && let Some(target) = cx.find(id)
        && target.is_player()
        && (target.flags.contains(CandidateFlags::CREATIVE)
            || target.flags.contains(CandidateFlags::SPECTATOR))
    {
        cx.set_target(None);
    }
    cx.intents.melee = Some(MeleeIntent::Stop);
    cx.intents.pose = Some(
        cx.intents
            .pose
            .unwrap_or_default()
            .merged(PoseIntent::clear(PoseBits::AGGRESSIVE)),
    );
    cx.clear_path();
}
