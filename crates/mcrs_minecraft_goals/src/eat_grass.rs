use mcrs_minecraft_brain::PoseBits;
use mcrs_minecraft_brain::PoseIntent;
use mcrs_minecraft_goal_world::CellFlags;
use mcrs_voxel::BlockPos;

use crate::BrainWorld;
use crate::EAT_WAIT;
use crate::EatState;
use crate::query::Cx;

const BABY_CHANCE: f64 = 1.0 / 25.0;
const ADULT_CHANCE: f64 = 1.0 / 500.0;
const EAT_UNITS: u8 = 20;

pub(crate) fn can_start<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &mut EatState,
) -> bool {
    let chance = if cx.me().is_baby() {
        BABY_CHANCE
    } else {
        ADULT_CHANCE
    };
    if !state.gate.passes(cx.turn, EAT_WAIT, chance) {
        cx.wake.due_at(state.gate.due());
        return false;
    }
    cx.wake.busy = true;
    let at = BlockPos::from(cx.me().position);
    let cell = cx.turn.world.cell(at);
    cell.flags.contains(CellFlags::EDIBLE) || cell.flags.contains(CellFlags::GRASS_BELOW)
}

pub(crate) fn start<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut EatState) {
    state.timer = EAT_UNITS;
    cx.intents.pose = Some(PoseIntent::set(PoseBits::EATING));
    cx.clear_path();
}

pub(crate) fn tick(state: &mut EatState) {
    state.timer = state.timer.saturating_sub(1);
}

pub(crate) fn can_continue<W: BrainWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &EatState,
) -> bool {
    cx.busy();
    state.timer > 0
}

pub(crate) fn stop<W: BrainWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut EatState) {
    state.timer = 0;
    cx.intents.pose = Some(
        cx.intents
            .pose
            .unwrap_or_default()
            .merged(PoseIntent::clear(PoseBits::EATING)),
    );
}
