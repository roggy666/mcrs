use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::query::Changed;
use bevy_ecs::query::Has;
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::ParamSet;
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
use mcrs_minecraft_engine::coordinator::clock::GameTime;
use mcrs_minecraft_engine_damage::DamageRequest;
use mcrs_minecraft_engine_damage::PendingHits;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_entity_motion::StepOutcome;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_engine_world::birth_clock::BirthClock;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_entity::damage_type_keys::IN_FIRE;
use mcrs_minecraft_entity::damage_type_keys::LAVA;
use mcrs_minecraft_entity::damage_type_keys::ON_FIRE;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::KeyedStream;
use mcrs_minecraft_entity_key::WriterKey;
use mcrs_minecraft_physics::Contact;
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
use crate::work::FireWork;

const ON_FIRE_AMOUNT: f32 = 1.0;
const FIRE_AMOUNT: f32 = 1.0;
const SOUL_FIRE_AMOUNT: f32 = 2.0;
const LAVA_AMOUNT: f32 = 4.0;
const FIRE_SECONDS: f32 = 8.0;
const LAVA_SECONDS: f32 = 15.0;
const DAMAGE_TYPE: ResourceLocation<'static> = rl!("minecraft:damage_type");

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FireDamageTypes {
    pub on_fire: Option<u32>,
    pub in_fire: Option<u32>,
    pub lava: Option<u32>,
}

pub(crate) fn bake_fire_damage_types(
    registries: Res<CurrentRegistries>,
    mut types: ResMut<FireDamageTypes>,
) {
    let position = |location: ResourceLocation<'_>| {
        registries
            .names_of(DAMAGE_TYPE)
            .and_then(|names| names.iter().position(|name| *name == location))
            .and_then(|index| u32::try_from(index).ok())
    };
    *types = FireDamageTypes {
        on_fire: position(ON_FIRE.location()),
        in_fire: position(IN_FIRE.location()),
        lava: position(LAVA.location()),
    };
}

type Burning<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Fire,
        &'static Exposure,
        Option<&'static Mob>,
        Option<&'static MotionState>,
        Option<&'static Transform>,
        Option<&'static EntityUuid>,
        Has<SunSensitive>,
    ),
>;

type Touching<'w, 's> = Query<'w, 's, (Entity, &'static MotionState, &'static StepOutcome)>;

type Lit<'w, 's> = Query<'w, 's, (Entity, &'static Fire), Changed<Fire>>;

type SunCandidates<'w, 's> = Query<'w, 's, (Entity, &'static Exposure), With<SunSensitive>>;

type PlayerCandidates<'w, 's> = Query<'w, 's, (Entity, &'static Exposure), With<Player>>;

struct Tending<'a, 'w> {
    pending: &'a mut PendingHits,
    changes: &'a mut MessageWriter<'w, ColumnChange>,
    streams: Option<&'a ColumnStreams>,
    types: FireDamageTypes,
    tick: u64,
    game_tick: u64,
    sun_chance: f64,
}

impl Tending<'_, '_> {
    fn hit(&mut self, target: Entity, key: EntityUuid, damage_type: Option<u32>, amount: f32) {
        let Some(damage_type) = damage_type else {
            return;
        };
        self.pending.push(DamageRequest::new(
            target,
            Hit::new(amount, DamageSource::of(damage_type)),
            WriterKey::Entity { key },
            self.tick,
            0,
        ));
    }
}

struct Tended {
    entity: Entity,
    key: EntityUuid,
    player: bool,
    fire_immune: bool,
    contact: Contact,
    position: DVec3,
}

fn gather(
    work: &mut FireWork,
    tick: u64,
    daylight: &Daylight,
    touching: &Touching<'_, '_>,
    lit: &Lit<'_, '_>,
    sun: &SunCandidates<'_, '_>,
    players: &PlayerCandidates<'_, '_>,
) {
    for (entity, fire) in lit {
        if fire.burning() {
            work.insert(entity);
        }
    }
    for (entity, state, outcome) in touching {
        if outcome.is_of(tick)
            && state
                .0
                .contact
                .intersects(Contact::LAVA | Contact::FIRE | Contact::SOUL_FIRE)
        {
            work.insert(entity);
        }
    }
    if daylight.bright {
        for (entity, exposure) in sun {
            if exposure.in_sun {
                work.insert(entity);
            }
        }
    }
    for (entity, exposure) in players {
        if exposure.in_lava || exposure.in_fire || exposure.in_soul_fire {
            work.insert(entity);
        }
    }
}

fn player_contact(exposure: &Exposure) -> Contact {
    let mut contact = Contact::empty();
    contact.set(Contact::WATER, exposure.in_water);
    contact.set(Contact::LAVA, exposure.in_lava);
    contact.set(Contact::FIRE, exposure.in_fire);
    contact.set(Contact::SOUL_FIRE, exposure.in_soul_fire);
    contact
}

fn ignite_by_contact(fire: &mut Fire, tended: &Tended, cx: &mut Tending<'_, '_>) -> bool {
    let mut ignited = false;
    if tended.contact.in_fire() {
        if !tended.fire_immune {
            if fire.ticks < 0 {
                fire.ticks += 1;
            } else if tended.player {
                let stream = KeyedStream::new(&tended.key, cx.game_tick);
                fire.ticks += 1 + stream.below(2).cast_signed();
            }
            if fire.ticks >= 0 {
                fire.ignite_seconds(FIRE_SECONDS);
                ignited = true;
            }
        }
        let amount = if tended.contact.contains(Contact::SOUL_FIRE) {
            SOUL_FIRE_AMOUNT
        } else {
            FIRE_AMOUNT
        };
        cx.hit(tended.entity, tended.key, cx.types.in_fire, amount);
    }
    if tended.contact.in_lava() && !tended.fire_immune {
        fire.ignite_seconds(LAVA_SECONDS);
        ignited = true;
        cx.hit(tended.entity, tended.key, cx.types.lava, LAVA_AMOUNT);
    }
    ignited
}

fn extinguish(fire: &mut Fire, tended: &Tended, cx: &mut Tending<'_, '_>) {
    if !fire.clear() {
        return;
    }
    let stream = KeyedStream::new(&tended.key, cx.game_tick);
    let source = if tended.player {
        SoundSource::Players
    } else {
        SoundSource::Neutral
    };
    play(
        SoundAt::extinguish(tended.position, source, &stream),
        cx.game_tick,
        cx.streams,
        cx.changes,
    );
}

fn tend(
    fire: &mut Fire,
    tended: &Tended,
    exposure: &Exposure,
    sun_sensitive: bool,
    cx: &mut Tending<'_, '_>,
) -> bool {
    let outcome = fire.tick(tended.fire_immune, tended.contact.in_lava());
    if outcome == (FireTick::Burning { hit: true }) {
        cx.hit(tended.entity, tended.key, cx.types.on_fire, ON_FIRE_AMOUNT);
    }
    let mut ignited = ignite_by_contact(fire, tended, cx);
    if sun_sensitive && exposure.in_sun {
        let stream = KeyedStream::new(&tended.key, cx.game_tick);
        if stream.unit(0) < cx.sun_chance {
            fire.ignite_seconds(SUN_BURN_SECONDS);
            ignited = true;
        }
    }
    if tended.contact.in_water() || exposure.in_rain {
        extinguish(fire, tended, cx);
    }
    if ignited || fire.burning() || tended.contact.in_fire() {
        return true;
    }
    let grace = if tended.player { Fire::PLAYER_GRACE } else { 0 };
    if fire.ticks != -grace {
        fire.grace(grace);
    }
    false
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each query, resource and writer is its own parameter"
)]
pub(crate) fn burn(
    clock: Res<BirthClock>,
    time: Res<GameTime>,
    daylight: Res<Daylight>,
    types: Res<FireDamageTypes>,
    streams: Query<&ColumnStreams>,
    touching: Touching,
    sun: SunCandidates,
    players: PlayerCandidates,
    mut work: ResMut<FireWork>,
    mut pending: ResMut<PendingHits>,
    mut changes: MessageWriter<ColumnChange>,
    mut fires: ParamSet<(Lit, Burning)>,
) {
    gather(
        &mut work,
        clock.0,
        &daylight,
        &touching,
        &fires.p0(),
        &sun,
        &players,
    );
    if work.is_empty() {
        return;
    }
    let streams = streams.single().ok();
    let mut cx = Tending {
        pending: &mut pending,
        changes: &mut changes,
        streams,
        types: *types,
        tick: clock.0,
        game_tick: time.0.cast_unsigned(),
        sun_chance: sun_burn_chance(daylight.brightness),
    };
    let mut entities = fires.p1();
    work.retain(|&entity| {
        let Ok((mut fire, exposure, mob, state, transform, uuid, sun_sensitive)) =
            entities.get_mut(entity)
        else {
            return false;
        };
        let Some(key) = state.map(|state| state.0.key).or(uuid.copied()) else {
            return false;
        };
        let tended = Tended {
            entity,
            key,
            player: mob.is_none(),
            fire_immune: mob.is_some_and(|mob| mob.kind.fire_immune),
            contact: state.map_or_else(|| player_contact(exposure), |state| state.0.contact),
            position: state
                .map(|state| state.0.body.position)
                .or(transform.map(|transform| transform.translation))
                .unwrap_or(DVec3::ZERO),
        };
        tend(&mut fire, &tended, exposure, sun_sensitive, &mut cx)
    });
}
