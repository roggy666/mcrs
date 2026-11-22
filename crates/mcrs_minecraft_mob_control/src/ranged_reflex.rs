#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangedReflex {
    pub speed_modifier: f32,
    pub min_interval: u16,
    pub max_interval: u16,
    pub range: f32,
    see_count: u16,
    attack_time: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangedStep {
    Wait,
    Attack { power: f32 },
}

const SEEN_FOR_STOP: u16 = 5;

impl RangedReflex {
    #[must_use]
    pub const fn new(
        speed_modifier: f32,
        min_interval: u16,
        max_interval: u16,
        range: f32,
    ) -> Self {
        Self {
            speed_modifier,
            min_interval,
            max_interval,
            range,
            see_count: 0,
            attack_time: -1,
        }
    }

    #[must_use]
    pub const fn holds_position(&self, distance_squared: f64) -> bool {
        let range = self.range as f64;
        distance_squared <= range * range && self.see_count >= SEEN_FOR_STOP
    }

    pub fn tick(&mut self, sees: bool, distance_squared: f64) -> RangedStep {
        if sees {
            self.see_count = self.see_count.saturating_add(1);
        } else {
            self.see_count = 0;
        }
        let fraction = (libm::sqrt(distance_squared) / f64::from(self.range)) as f32;
        let min = f32::from(self.min_interval);
        let max = f32::from(self.max_interval);
        self.attack_time = self.attack_time.saturating_sub(1);
        if self.attack_time == 0 {
            if !sees {
                return RangedStep::Wait;
            }
            let power = fraction.clamp(0.1, 1.0);
            self.attack_time = (fraction * (max - min) + min) as i32;
            return RangedStep::Attack { power };
        }
        if self.attack_time < 0 {
            self.attack_time = (min + (max - min) * fraction) as i32;
        }
        RangedStep::Wait
    }
}
