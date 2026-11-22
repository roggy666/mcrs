use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::query::Has;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use glam::DVec3;
use mcrs_minecraft_damage::DamageSource;
use mcrs_minecraft_damage::Exposure;
use mcrs_minecraft_damage::Fire;
use mcrs_minecraft_damage::FireTick;
use mcrs_minecraft_damage::Hit;
use mcrs_minecraft_engine::column_change::ColumnChange;
use mcrs_minecraft_engine::column_streams::ColumnStreams;
use mcrs_minecraft_engine_damage::DamageRequest;
use mcrs_minecraft_engine_damage::PendingHits;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_engine_world::birth_clock::BirthClock;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_entity::damage_type_keys::ON_FIRE;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::KeyedStream;
use mcrs_minecraft_entity_key::WriterKey;
use mcrs_minecraft_protocol_game::sound::SoundSource;
use mcrs_minecraft_registries::CurrentRegistries;
use mcrs_minecraft_resource::ResourceLocation;
use mcrs_minecraft_resource::rl;

use crate::exposure::Daylight;
use crate::sounds::SoundAt;
use crate::sounds::play;
use crate::sun::SUN_BURN_SECONDS;
use crate::sun::SunSensitive;
use crate::sun::sun_burn_chance;

const ON_FIRE_AMOUNT: f32 = 1.0;
const DAMAGE_TYPE: ResourceLocation<'static> = rl!("minecraft:damage_type");

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FireDamageType(pub Option<u32>);

pub(crate) fn bake_fire_damage_type(
    registries: Res<CurrentRegistries>,
    mut damage_type: ResMut<FireDamageType>,
) {
    damage_type.0 = registries
        .names_of(DAMAGE_TYPE)
        .and_then(|names| names.iter().position(|name| *name == ON_FIRE.location()))
        .and_then(|index| u32::try_from(index).ok());
}

type Burning<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Fire,
        &'static Exposure,
        Option<&'static Mob>,
        Option<&'static MotionState>,
        Option<&'static Transform>,
        Option<&'static EntityUuid>,
        Has<SunSensitive>,
    ),
>;

pub(crate) fn burn(
    clock: Res<BirthClock>,
    daylight: Res<Daylight>,
    damage_type: Res<FireDamageType>,
    streams: Query<&ColumnStreams>,
    mut pending: ResMut<PendingHits>,
    mut changes: MessageWriter<ColumnChange>,
    mut entities: Burning,
) {
    let tick = clock.0;
    let streams = streams.single().ok();
    let chance = sun_burn_chance(daylight.brightness);
    for (entity, mut fire, exposure, mob, state, transform, uuid, sun_sensitive) in &mut entities {
        let Some(key) = state.map(|state| state.0.key).or(uuid.copied()) else {
            continue;
        };
        let position = state
            .map(|state| state.0.body.position)
            .or(transform.map(|transform| transform.translation))
            .unwrap_or(DVec3::ZERO);
        let fire_immune = mob.is_some_and(|mob| mob.kind.fire_immune);
        let stream = KeyedStream::new(&key, daylight.tick);
        if fire.ticks != 0 || fire_immune {
            let outcome = fire.tick(fire_immune, false);
            if outcome == (FireTick::Burning { hit: true })
                && let Some(damage_type) = damage_type.0
            {
                pending.push(DamageRequest::new(
                    entity,
                    Hit::new(ON_FIRE_AMOUNT, DamageSource::of(damage_type)),
                    WriterKey::Entity { key },
                    tick,
                    0,
                ));
            }
        }
        let mut ignited = false;
        if sun_sensitive && exposure.in_sun && stream.unit(0) < chance {
            fire.ignite_seconds(SUN_BURN_SECONDS);
            ignited = true;
        }
        if exposure.wet() && fire.burning() {
            fire.clear();
            play(
                SoundAt::extinguish(
                    position,
                    if mob.is_some() {
                        SoundSource::Neutral
                    } else {
                        SoundSource::Players
                    },
                    &stream,
                ),
                tick,
                streams,
                &mut changes,
            );
        }
        if !ignited && !fire.burning() {
            let grace = if mob.is_some() { 0 } else { Fire::PLAYER_GRACE };
            if fire.ticks != -grace {
                fire.grace(grace);
            }
        }
    }
}
