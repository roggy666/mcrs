use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::RangedIntent;

use crate::BrainWorld;
use crate::RangedState;
use crate::query::Cx;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RangedParams {
    pub(crate) speed: f32,
    pub(crate) min_interval: u16,
    pub(crate) max_interval: u16,
    pub(crate) range: f32,
}

pub(crate) fn can_start<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &mut RangedState,
) -> bool {
    let Some(target) = cx.attack_target() else {
        return false;
    };
    cx.busy();
    state.target = EntityRef::of(target.id);
    true
}

pub(crate) fn start<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, params: &RangedParams) {
    cx.intents.ranged = Some(RangedIntent::Start {
        speed_modifier: params.speed,
        min_interval: params.min_interval,
        max_interval: params.max_interval,
        range: params.range,
    });
}

pub(crate) fn tick<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    params: &RangedParams,
    state: &RangedState,
) {
    let Some(target) = state.target.get().and_then(|id| cx.find(id)) else {
        return;
    };
    let range = f64::from(params.range);
    if cx.feet_distance_squared(&target) <= range * range && cx.sees(&target) {
        cx.clear_path();
    } else {
        cx.request_path_to(&target, params.speed);
    }
}

pub(crate) fn can_continue<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &mut RangedState,
) -> bool {
    cx.busy();
    if let Some(target) = cx.attack_target() {
        state.target = EntityRef::of(target.id);
        return true;
    }
    state
        .target
        .get()
        .and_then(|id| cx.find(id))
        .is_some_and(|target| target.is_alive())
        && cx.path_running()
}

pub(crate) fn stop<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut RangedState) {
    state.target = EntityRef::NONE;
    cx.intents.ranged = Some(RangedIntent::Stop);
}
