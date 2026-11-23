use glam::DVec3;
use mcrs_minecraft_numeric as numeric;

use crate::BodyFlags;
use crate::ControlInput;
use crate::Wanted;
use crate::angles;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoveControl {
    operation: Operation,
    flight: Option<FlightControl>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlightControl {
    pub hovers: bool,
    pub can_float: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Operation {
    #[default]
    Wait,
    MoveTo(Wanted),
    Jump {
        speed_modifier: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveStep {
    pub forward: f32,
    pub vertical: f32,
    pub speed: Option<f32>,
    pub yaw: Option<f32>,
    pub jump: bool,
    pub no_gravity: Option<bool>,
}

const AT_WANTED: f64 = 2.500_000_3e-7_f32 as f64;
const NEAR: f64 = 1.0e-5_f32 as f64;
const LARGEST_TURN: f32 = 90.0;

impl MoveControl {
    #[must_use]
    pub const fn flying(flight: FlightControl) -> Self {
        Self {
            operation: Operation::Wait,
            flight: Some(flight),
        }
    }

    #[must_use]
    pub const fn flight(&self) -> Option<FlightControl> {
        self.flight
    }

    pub fn set_wanted(&mut self, wanted: Wanted) {
        if !matches!(self.operation, Operation::Jump { .. }) {
            self.operation = Operation::MoveTo(wanted);
        }
    }

    pub fn step(&mut self, input: &ControlInput) -> MoveStep {
        let rest = MoveStep {
            forward: 0.0,
            vertical: 0.0,
            speed: None,
            yaw: None,
            jump: false,
            no_gravity: None,
        };
        if let Some(flight) = self.flight {
            return self.fly(input, flight, rest);
        }
        match self.operation {
            Operation::Wait => rest,
            Operation::Jump { speed_modifier } => {
                let speed = mob_speed(speed_modifier, input.movement_speed);
                let flags = input.flags;
                if flags.contains(BodyFlags::ON_GROUND) || flags.contains(BodyFlags::IN_LIQUID) {
                    self.operation = Operation::Wait;
                }
                MoveStep {
                    forward: speed,
                    speed: Some(speed),
                    ..rest
                }
            }
            Operation::MoveTo(wanted) => {
                self.operation = Operation::Wait;
                self.move_to(input, wanted).unwrap_or(rest)
            }
        }
    }

    fn fly(&mut self, input: &ControlInput, flight: FlightControl, rest: MoveStep) -> MoveStep {
        let Operation::MoveTo(wanted) = self.operation else {
            return MoveStep {
                no_gravity: (!flight.hovers).then_some(false),
                ..rest
            };
        };
        self.operation = Operation::Wait;
        let rest = MoveStep {
            no_gravity: Some(true),
            ..rest
        };
        let position: DVec3 = input.position;
        let dx = wanted.position.x - position.x;
        let dy = wanted.position.y - position.y;
        let dz = wanted.position.z - position.z;
        let horizontal = dx * dx + dz * dz;
        if horizontal + dy * dy < AT_WANTED {
            return rest;
        }
        let yaw = turned_yaw(input.rotations.yaw, dx, dz);
        let base = if input.flags.contains(BodyFlags::ON_GROUND) {
            input.movement_speed
        } else {
            input.flying_speed
        };
        let speed = mob_speed(wanted.speed_modifier, base);
        let flat = horizontal.sqrt();
        let vertical = if dy.abs() > NEAR || flat.abs() > NEAR {
            if dy > 0.0 { speed } else { -speed }
        } else {
            0.0
        };
        MoveStep {
            forward: speed,
            vertical,
            speed: Some(speed),
            yaw: Some(yaw),
            ..rest
        }
    }

    fn move_to(&mut self, input: &ControlInput, wanted: Wanted) -> Option<MoveStep> {
        let position: DVec3 = input.position;
        let dx = wanted.position.x - position.x;
        let dy = wanted.position.y - position.y;
        let dz = wanted.position.z - position.z;
        let horizontal = dx * dx + dz * dz;
        if horizontal + dy * dy < AT_WANTED {
            return None;
        }
        let turned = turned_yaw(input.rotations.yaw, dx, dz);
        let speed = mob_speed(wanted.speed_modifier, input.movement_speed);
        let body = input.body;
        let climbs =
            dy > f64::from(body.step_height) && horizontal < f64::from(body.width.max(1.0));
        let blocked = input.feet_block_top.is_some_and(|top| position.y < top);
        let jump = climbs || blocked;
        if jump {
            self.operation = Operation::Jump {
                speed_modifier: wanted.speed_modifier,
            };
        }
        Some(MoveStep {
            forward: speed,
            vertical: 0.0,
            speed: Some(speed),
            yaw: Some(turned),
            jump,
            no_gravity: None,
        })
    }
}

fn turned_yaw(yaw: f32, dx: f64, dz: f64) -> f32 {
    let turn =
        numeric::wrap_degrees(angles::yaw_to(dx, dz) - yaw).clamp(-LARGEST_TURN, LARGEST_TURN);
    let mut turned = yaw + turn;
    if turned < 0.0 {
        turned += 360.0;
    } else if turned > 360.0 {
        turned -= 360.0;
    }
    turned
}

#[must_use]
pub fn mob_speed(speed_modifier: f32, movement_speed: f64) -> f32 {
    (f64::from(speed_modifier) * movement_speed) as f32
}
