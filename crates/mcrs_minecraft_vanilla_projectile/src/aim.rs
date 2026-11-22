use glam::DVec3;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_entity_key::KeyedStream;

use crate::ProjectileKind;

const SPREAD: f64 = 0.017_227_5;
const LIFT: f64 = 0.2;
const MOB_SPEED: f64 = 1.6;
const ARROW_INACCURACY: f64 = 6.0;
const SNOWBALL_INACCURACY: f64 = 12.0;
const SNOWBALL_DROP: f64 = 1.1;
const START_BELOW_EYE: f64 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub kind: ProjectileKind,
    pub position: DVec3,
    pub velocity: DVec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shooter {
    pub entity_type: EntityType,
    pub feet: DVec3,
    pub eye_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aimed {
    pub feet: DVec3,
    pub height: f32,
    pub eye_height: f32,
}

#[must_use]
pub fn mob_shot(shooter: &Shooter, target: &Aimed, stream: &KeyedStream) -> Option<Shot> {
    let (kind, inaccuracy) = match shooter.entity_type {
        EntityType::Skeleton
        | EntityType::Stray
        | EntityType::Bogged
        | EntityType::Parched
        | EntityType::Pillager => (ProjectileKind::Arrow, ARROW_INACCURACY),
        EntityType::SnowGolem => (ProjectileKind::Snowball, SNOWBALL_INACCURACY),
        _ => return None,
    };
    let position = DVec3::new(
        shooter.feet.x,
        shooter.feet.y + f64::from(shooter.eye_height) - START_BELOW_EYE,
        shooter.feet.z,
    );
    let aim_y = match kind {
        ProjectileKind::Arrow => target.feet.y + f64::from(target.height) / 3.0,
        ProjectileKind::Snowball => target.feet.y + f64::from(target.eye_height) - SNOWBALL_DROP,
    };
    let to = DVec3::new(target.feet.x, aim_y, target.feet.z) - position;
    let horizontal = to.x.hypot(to.z);
    let direction = DVec3::new(to.x, to.y + horizontal * LIFT, to.z);
    let velocity = launch_velocity(direction, MOB_SPEED, inaccuracy, stream);
    Some(Shot {
        kind,
        position,
        velocity,
    })
}

#[must_use]
pub fn launch_velocity(
    direction: DVec3,
    speed: f64,
    inaccuracy: f64,
    stream: &KeyedStream,
) -> DVec3 {
    let unit = direction.normalize_or_zero();
    let mut draw = 0;
    let mut spread = || {
        let first = stream.unit(draw);
        let second = stream.unit(draw + 1);
        draw += 2;
        SPREAD * inaccuracy * (first - second)
    };
    let scattered = DVec3::new(unit.x + spread(), unit.y + spread(), unit.z + spread());
    scattered * speed
}
