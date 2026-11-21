use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::ItemTest;
use mcrs_minecraft_goal_world::GoalWorld;

use crate::TemptState;
use crate::query::Cx;

const STOP_DISTANCE_SQUARED: f64 = 2.5 * 2.5;
const CALM_DOWN_TICKS: u32 = 100;
const HEAD_SPEED_BONUS: f32 = 20.0;

pub(crate) fn can_start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    items: ItemTest,
    state: &mut TemptState,
) -> bool {
    let now = cx.tick() as u32;
    if now.wrapping_sub(state.calm_until) as i32 as i64 > 0 && state.calm_until != 0 {
        state.calm_until = 0;
    }
    if state.calm_until != 0 && now < state.calm_until {
        return false;
    }
    state.calm_until = 0;
    let range = cx.tempt_range();
    cx.collect_players();
    let tables = cx.turn.items;
    let player = cx.nearest(
        |cx, candidate| cx.feet_distance_squared(candidate),
        |cx, candidate| {
            let reach = (range * f64::from(candidate.visibility)).max(2.0);
            candidate.is_alive()
                && !candidate
                    .flags
                    .contains(mcrs_minecraft_brain::CandidateFlags::SPECTATOR)
                && candidate
                    .hands
                    .iter()
                    .any(|item| items.passes(tables, *item))
                && cx.feet_distance_squared(candidate) <= reach * reach
        },
    );
    if player.is_some() {
        cx.busy();
    }
    state.player = EntityRef::some(player.map(|candidate| candidate.id));
    player.is_some()
}

pub(crate) fn tick<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    speed: f32,
    state: &TemptState,
) {
    let Some(player) = state.player.get().and_then(|id| cx.find(id)) else {
        return;
    };
    let turn_speed = cx.head_yaw_limit() + HEAD_SPEED_BONUS;
    let pitch = cx.pitch_limit();
    cx.look_at(player.eye(), Some(turn_speed), Some(pitch));
    if cx.feet_distance_squared(&player) < STOP_DISTANCE_SQUARED {
        cx.clear_path();
    } else {
        cx.request_path_to(&player, speed);
    }
}

pub(crate) fn stop<W: GoalWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut TemptState) {
    state.player = EntityRef::NONE;
    cx.clear_path();
    state.calm_until = (cx.tick() as u32).wrapping_add(CALM_DOWN_TICKS).max(1);
}
