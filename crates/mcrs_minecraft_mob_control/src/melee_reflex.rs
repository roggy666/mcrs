use mcrs_minecraft_brain::Pursuit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeReflex {
    pub speed_modifier: f32,
    pub interval: u8,
    pub pursuit: Pursuit,
    cooldown: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeleeStep {
    Wait,
    Hit,
}

impl MeleeReflex {
    #[must_use]
    pub const fn new(speed_modifier: f32, interval: u8, pursuit: Pursuit) -> Self {
        Self {
            speed_modifier,
            interval,
            pursuit,
            cooldown: 0,
        }
    }

    #[must_use]
    pub const fn cooldown(&self) -> u8 {
        self.cooldown
    }

    pub fn tick(&mut self, in_reach: bool, sees: bool) -> MeleeStep {
        self.cooldown = self.cooldown.saturating_sub(1);
        if self.cooldown == 0 && in_reach && sees {
            self.cooldown = self.interval;
            MeleeStep::Hit
        } else {
            MeleeStep::Wait
        }
    }
}

pub const LOOK_TURN_SPEED: f32 = 30.0;
pub const LOOK_PITCH_LIMIT: f32 = 30.0;
