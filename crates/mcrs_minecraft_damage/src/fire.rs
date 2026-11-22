#[cfg(feature = "bevy")]
use bevy_ecs::component::Component;

const HIT_PERIOD: i32 = 20;
const TICKS_PER_SECOND: f32 = 20.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct Fire {
    pub ticks: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FireTick {
    Idle,
    Burning { hit: bool },
}

impl Fire {
    pub const PLAYER_GRACE: i32 = 20;

    #[must_use]
    pub const fn of(ticks: i32) -> Self {
        Self { ticks }
    }

    #[must_use]
    pub const fn burning(&self) -> bool {
        self.ticks > 0
    }

    pub fn ignite(&mut self, ticks: i32) {
        if self.ticks < ticks {
            self.ticks = ticks;
        }
    }

    pub fn ignite_seconds(&mut self, seconds: f32) {
        self.ignite(libm::floorf(seconds * TICKS_PER_SECOND) as i32);
    }

    pub fn clear(&mut self) -> bool {
        let was_burning = self.burning();
        self.ticks = self.ticks.min(0);
        was_burning
    }

    pub fn tick(&mut self, fire_immune: bool, in_lava: bool) -> FireTick {
        if self.ticks <= 0 {
            return FireTick::Idle;
        }
        if fire_immune {
            self.ticks = 0;
            return FireTick::Idle;
        }
        let hit = self.ticks % HIT_PERIOD == 0 && !in_lava;
        self.ticks -= 1;
        FireTick::Burning { hit }
    }

    pub fn grace(&mut self, ticks: i32) {
        if !self.burning() {
            self.ticks = -ticks;
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn an_ignition_of_eight_seconds_hits_eight_times_and_burns_out() {
        let mut fire = Fire::default();
        fire.ignite_seconds(8.0);
        assert_eq!(fire.ticks, 160);
        let mut hits = Vec::new();
        for tick in 1..=170 {
            if fire.tick(false, false) == (FireTick::Burning { hit: true }) {
                hits.push(tick);
            }
        }
        assert_eq!(hits, vec![1, 21, 41, 61, 81, 101, 121, 141]);
        assert!(!fire.burning());
    }

    #[test]
    fn an_ignition_never_lowers_the_ticks_and_a_clear_keeps_the_grace() {
        let mut fire = Fire::of(100);
        fire.ignite(40);
        assert_eq!(fire.ticks, 100);
        assert!(fire.clear());
        assert_eq!(fire.ticks, 0);
        fire.grace(Fire::PLAYER_GRACE);
        assert_eq!(fire.ticks, -20);
        assert!(!fire.clear());
        assert_eq!(fire.ticks, -20);
    }

    #[test]
    fn a_fire_immune_entity_loses_its_ticks_and_lava_skips_the_hit() {
        let mut fire = Fire::of(40);
        assert_eq!(fire.tick(true, false), FireTick::Idle);
        assert_eq!(fire.ticks, 0);
        let mut fire = Fire::of(40);
        assert_eq!(fire.tick(false, true), FireTick::Burning { hit: false });
        assert_eq!(fire.ticks, 39);
    }
}
