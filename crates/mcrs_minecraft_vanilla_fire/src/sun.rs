use bevy_ecs::component::Component;
use bevy_ecs::lifecycle::Add;
use bevy_ecs::observer::On;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_entity::entity_type_keys::EntityType;

pub const SUN_SENSITIVE: &[EntityType] = &[
    EntityType::Zombie,
    EntityType::ZombieVillager,
    EntityType::Drowned,
    EntityType::Skeleton,
    EntityType::Stray,
    EntityType::Bogged,
    EntityType::Parched,
    EntityType::Phantom,
];

pub const SUN_BURN_SECONDS: f32 = 8.0;
const LEAST_BRIGHTNESS: f64 = 0.5;
const BRIGHTNESS_OFFSET: f64 = 0.4;
const CHANCE_SCALE: f64 = 2.0 / 30.0;

#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct SunSensitive;

#[must_use]
pub fn sun_burn_chance(brightness: f64) -> f64 {
    if brightness > LEAST_BRIGHTNESS {
        (brightness - BRIGHTNESS_OFFSET) * CHANCE_SCALE
    } else {
        0.0
    }
}

pub(crate) fn mark_sun_sensitive(add: On<Add<Mob>>, mobs: Query<&Mob>, mut commands: Commands) {
    let entity = add.event().entity;
    let Ok(mob) = mobs.get(entity) else {
        return;
    };
    if SUN_SENSITIVE.contains(&mob.kind.entity_type) {
        commands.entity(entity).insert(SunSensitive);
    }
}
