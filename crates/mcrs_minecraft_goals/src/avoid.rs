use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_voxel::BlockPos;

use crate::AVOID_SEARCH;
use crate::AVOID_STRENGTH;
use crate::AvoidExtra;
use crate::AvoidState;
use crate::SearchRange;
use crate::query::Combat;
use crate::query::Cx;
use crate::query::Ray;
use crate::query::Visibility;
use crate::random_position;

const BOX_HEIGHT: f64 = 3.0;
const SPRINT_SQUARED: f64 = 49.0;
const AWAY_RANGE: SearchRange = SearchRange::new(16, 7);
const DEFAULT_LLAMA_STRENGTH: u32 = 1;
const STRENGTH_DRAWS: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AvoidParams {
    pub(crate) kinds: KindSet,
    pub(crate) distance: f32,
    pub(crate) walk: f32,
    pub(crate) sprint: f32,
    pub(crate) extra: AvoidExtra,
}

pub(crate) fn can_start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    params: &AvoidParams,
    state: &mut AvoidState,
) -> bool {
    if params.extra == AvoidExtra::LlamaStrength && cx.me().flags.contains(MobFlags::TAME) {
        return false;
    }
    if !cx.turn.neighbours.has_kind(&params.kinds) {
        return false;
    }
    cx.busy();
    let distance = f64::from(params.distance);
    let (min, max) = cx.grown_box(distance, BOX_HEIGHT);
    cx.collect_mobs(min, max, &params.kinds);
    let combat = Combat {
        range: distance,
        ray: Ray::Check,
        visibility: Visibility::Use,
    };
    let Some(entity) = cx.nearest_sorted(
        |cx, candidate| cx.feet_distance_squared(candidate),
        |cx, candidate| cx.combat_test(candidate, combat),
    ) else {
        return false;
    };
    if params.extra == AvoidExtra::LlamaStrength
        && cx.turn.random(AVOID_STRENGTH).below(STRENGTH_DRAWS) > DEFAULT_LLAMA_STRENGTH
    {
        return false;
    }
    let feet = cx.me().position;
    let Some(destination) = random_position::away_search(
        cx.turn.world,
        feet,
        entity.position,
        AWAY_RANGE,
        &mut cx.turn.random(AVOID_SEARCH),
    ) else {
        return false;
    };
    if entity.position.distance_squared(destination) < entity.position.distance_squared(feet) {
        return false;
    }
    if cx
        .turn
        .world
        .route(feet, BlockPos::from(destination))
        .is_none()
    {
        return false;
    }
    state.entity = EntityRef::of(entity.id);
    state.destination = BlockPos::from(destination);
    true
}

pub(crate) fn start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    params: &AvoidParams,
    state: &AvoidState,
) {
    if params.extra == AvoidExtra::LlamaStrength {
        cx.set_target(None);
    }
    cx.request_path(state.destination.bottom_centre(), params.walk);
}

pub(crate) fn tick<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    params: &AvoidParams,
    state: &AvoidState,
) {
    if params.extra == AvoidExtra::LlamaStrength {
        cx.set_target(None);
    }
    let near = state
        .entity
        .get()
        .and_then(|id| cx.find(id))
        .is_some_and(|entity| cx.feet_distance_squared(&entity) < SPRINT_SQUARED);
    cx.intents.speed = Some(if near { params.sprint } else { params.walk });
}

pub(crate) fn can_continue<W: GoalWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) -> bool {
    cx.busy();
    cx.path_running()
}

pub(crate) fn stop(state: &mut AvoidState) {
    state.entity = EntityRef::NONE;
}
