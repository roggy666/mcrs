use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_goal_world::GoalWorld;

use crate::FollowState;
use crate::Kinds;
use crate::breed::kind_set;
use crate::query::Cx;

const ADULT_RANGE_XZ: f64 = 8.0;
const ADULT_RANGE_Y: f64 = 4.0;
const START_DISTANCE_SQUARED: f64 = 9.0;
const STOP_DISTANCE_SQUARED: f64 = 256.0;
const PATH_UNITS: u8 = 5;

pub(crate) fn can_start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    kinds: Kinds,
    state: &mut FollowState,
) -> bool {
    let me = *cx.me();
    if !me.is_baby() {
        return false;
    }
    let set = kind_set(kinds, me.kind);
    let (min, max) = cx.grown_box(ADULT_RANGE_XZ, ADULT_RANGE_Y);
    cx.collect_mobs(min, max, &set);
    let adult = cx.nearest(
        |cx, candidate| cx.feet_distance_squared(candidate),
        |_, candidate| !candidate.is_baby(),
    );
    let Some(adult) = adult else {
        state.adult = EntityRef::NONE;
        return false;
    };
    if cx.feet_distance_squared(&adult) < START_DISTANCE_SQUARED {
        state.adult = EntityRef::NONE;
        return false;
    }
    state.adult = EntityRef::of(adult.id);
    true
}

pub(crate) fn start(state: &mut FollowState) {
    state.timer = 0;
}

pub(crate) fn tick<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    speed: f32,
    state: &mut FollowState,
) {
    state.timer = state.timer.saturating_sub(1);
    if state.timer > 0 {
        return;
    }
    state.timer = PATH_UNITS;
    if let Some(adult) = state.adult.get().and_then(|id| cx.find(id)) {
        cx.request_path_to(&adult, speed);
    }
}

pub(crate) fn can_continue<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &FollowState,
) -> bool {
    if !cx.me().is_baby() {
        return false;
    }
    let Some(adult) = state.adult.get().and_then(|id| cx.find(id)) else {
        return false;
    };
    let distance = cx.feet_distance_squared(&adult);
    adult.is_alive() && (START_DISTANCE_SQUARED..=STOP_DISTANCE_SQUARED).contains(&distance)
}

pub(crate) fn stop(state: &mut FollowState) {
    state.adult = EntityRef::NONE;
}
