use mcrs_minecraft_brain::Leap;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_goal_world::GoalWorld;

use crate::Gate;
use crate::LEAP_WAIT;
use crate::query::Cx;

const NEAR_SQUARED: f64 = 4.0;
const FAR_SQUARED: f64 = 16.0;
const CHANCE: f64 = 1.0 / 3.0;

pub(crate) fn can_start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    gate: &mut Gate,
) -> bool {
    let me = *cx.me();
    if me.flags.contains(MobFlags::CONTROLLED) {
        gate.end();
        return false;
    }
    let Some(target) = cx.attack_target() else {
        gate.end();
        return false;
    };
    let distance = cx.feet_distance_squared(&target);
    if !(NEAR_SQUARED..=FAR_SQUARED).contains(&distance) || !me.flags.contains(MobFlags::ON_GROUND)
    {
        gate.end();
        return false;
    }
    cx.busy();
    gate.passes(cx.turn, LEAP_WAIT, CHANCE)
}

pub(crate) fn start<W: GoalWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, vertical: f32) {
    let Some(target) = cx.attack_target() else {
        return;
    };
    let dx = target.position.x - cx.me().position.x;
    let dz = target.position.z - cx.me().position.z;
    let length = libm::sqrt(dx * dx + dz * dz);
    let (toward_x, toward_z) = if length * length > 1.0e-7 {
        ((dx / length) as f32, (dz / length) as f32)
    } else {
        (0.0, 0.0)
    };
    cx.intents.leap = Some(Leap {
        toward_x,
        toward_z,
        vertical,
    });
}

pub(crate) fn can_continue<W: GoalWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>) -> bool {
    cx.busy();
    !cx.me().flags.contains(MobFlags::ON_GROUND)
}
