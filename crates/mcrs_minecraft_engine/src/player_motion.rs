use bevy_ecs::component::Component;
use bevy_ecs::system::Query;
use bevy_math::DVec3;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerMotion {
    delta: DVec3,
    fresh: bool,
}

impl PlayerMotion {
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
