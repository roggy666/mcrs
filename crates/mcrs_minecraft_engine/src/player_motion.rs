use bevy_ecs::component::Component;
use bevy_ecs::system::Query;
use bevy_math::DVec3;

/// The movement of the last movement packet of a player, in blocks per tick.
///
/// A movement packet records its position delta (zero for a packet without a position). A tick
/// without a movement packet settles the delta to zero at its end. Thus a reader of the next tick
/// sees the movement of the last client tick, or zero when the client sent nothing.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerMotion {
    delta: DVec3,
    fresh: bool,
}

impl PlayerMotion {
    /// The movement of the last movement packet, or zero when the last tick had no packet.
    #[must_use]
    pub fn delta(&self) -> DVec3 {
        self.delta
    }

    pub fn record(&mut self, delta: DVec3) {
        self.delta = delta;
        self.fresh = true;
    }

    fn settle(&mut self) {
        if !self.fresh {
            self.delta = DVec3::ZERO;
        }
        self.fresh = false;
    }
}

/// Settles each motion at the end of the tick: a motion that no packet refreshed becomes zero.
pub(crate) fn settle_player_motion(mut motions: Query<&mut PlayerMotion>) {
    for mut motion in &mut motions {
        if motion.fresh || motion.delta != DVec3::ZERO {
            motion.settle();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recorded_delta_lasts_one_settle_and_then_becomes_zero() {
        let mut motion = PlayerMotion::default();
        motion.record(DVec3::new(0.2, 0.0, -0.1));
        motion.settle();
        assert_eq!(motion.delta(), DVec3::new(0.2, 0.0, -0.1));
        motion.settle();
        assert_eq!(motion.delta(), DVec3::ZERO);
    }
}
