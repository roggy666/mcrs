use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use glam::DVec3;
use mcrs_minecraft_damage::DamageSource;
use mcrs_minecraft_damage::DamageTypeRules;
use mcrs_minecraft_damage::Defence;
use mcrs_minecraft_damage::Hit;
use mcrs_minecraft_damage::HitOutcome;
use mcrs_minecraft_damage::HurtRules;
use mcrs_minecraft_damage::Vitals;
use mcrs_minecraft_damage::preview;
use mcrs_minecraft_engine::PlayerBody;
use mcrs_minecraft_engine::column_change::ColumnChange;
use mcrs_minecraft_engine::column_streams::ColumnStreams;
use mcrs_minecraft_engine_damage::DamageRequest;
use mcrs_minecraft_engine_damage::PendingHits;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_entity_motion::StepOutcome;
use mcrs_minecraft_engine_physics::EntityEffect;
use mcrs_minecraft_engine_physics::EntityWrite;
use mcrs_minecraft_engine_physics::EntityWrites;
use mcrs_minecraft_engine_physics::MobIndex;
use mcrs_minecraft_engine_world::birth_clock::BirthClock;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_entity::damage_type_keys::ARROW;
use mcrs_minecraft_entity::damage_type_keys::THROWN;
use mcrs_minecraft_entity_key::Cause;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::KeyedStream;
use mcrs_minecraft_entity_key::WriterKey;
use mcrs_minecraft_game_mode::GameMode;
use mcrs_minecraft_game_mode::PlayerGameMode;
use mcrs_minecraft_physics::Aabb;
use mcrs_minecraft_physics::BodyGrid;
use mcrs_minecraft_physics::GridFlags;
use mcrs_minecraft_physics::segment_meets;
use mcrs_minecraft_registries::CurrentRegistries;
use mcrs_minecraft_resource::ResourceLocation;
use mcrs_minecraft_resource::rl;

use crate::Projectile;
use crate::ProjectileKind;
use crate::pickup::RestingArrows;
use crate::sounds::SoundAt;
use crate::sounds::play;

const DAMAGE_TYPE: ResourceLocation<'static> = rl!("minecraft:damage_type");
const BOUNCE: f64 = -0.1;
const OWNER_REACH: f64 = 1.0;
const SNOWBALL_AMOUNT: f32 = 0.0;
const KNOCKBACK_POWER: f64 = 0.4;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct ProjectileDamageTypes {
    arrow: Option<u32>,
    thrown: Option<u32>,
}

pub(crate) fn bake_damage_types(
    registries: Res<CurrentRegistries>,
    mut types: ResMut<ProjectileDamageTypes>,
) {
    let position = |location: ResourceLocation<'_>| {
        registries
            .names_of(DAMAGE_TYPE)
            .and_then(|names| names.iter().position(|name| *name == location))
            .and_then(|index| u32::try_from(index).ok())
    };
    *types = ProjectileDamageTypes {
        arrow: position(ARROW.location()),
        thrown: position(THROWN.location()),
    };
}

type Flying<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static MotionState,
        &'static StepOutcome,
        &'static mut Projectile,
    ),
>;

type Targets<'w, 's> = Query<'w, 's, (&'static Vitals, Option<&'static Defence>)>;

type Players<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static EntityUuid,
        &'static Transform,
        &'static PlayerGameMode,
        PlayerBody,
    ),
    With<Player>,
>;

#[derive(Clone, Copy, Debug)]
struct Candidate {
    key: EntityUuid,
    entity: Option<Entity>,
    bounds: Aabb,
    centre: DVec3,
}

#[derive(Clone, Copy, Debug)]
struct EntityHit {
    rank: f64,
    entry: f64,
    candidate: Candidate,
}

struct FlightWorld<'w, 's, 'a> {
    grid: &'a BodyGrid,
    players: &'a Players<'w, 's>,
    index: &'a MobIndex,
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each resource, query and writer is its own parameter"
)]
pub(crate) fn fly_projectiles(
    mut projectiles: Flying,
    targets: Targets,
    players: Players,
    grid: Res<BodyGrid>,
    index: Res<MobIndex>,
    clock: Res<BirthClock>,
    types: Res<ProjectileDamageTypes>,
    rules: Res<DamageTypeRules>,
    streams: Query<&ColumnStreams>,
    mut pending: ResMut<PendingHits>,
    mut writes: ResMut<EntityWrites>,
    mut resting: ResMut<RestingArrows>,
    mut changes: MessageWriter<ColumnChange>,
    mut commands: Commands,
) {
    let tick = clock.0;
    let streams = streams.single().ok();
    let world = FlightWorld {
        grid: &grid,
        players: &players,
        index: &index,
    };
    for (entity, motion, outcome, mut projectile) in &mut projectiles {
        if !outcome.is_of(tick) {
            continue;
        }
        let start = outcome.start;
        let state = &motion.0;
        let end = state.body.position;
        if projectile.in_ground {
            if end == start && !outcome.kernel.stuck_in_block {
                projectile.shake = projectile.shake.saturating_sub(1);
                projectile.despawn = projectile.despawn.saturating_add(1);
                if projectile.despawn >= Projectile::DESPAWN_TICKS {
                    commands.entity(entity).despawn();
                }
                continue;
            }
            projectile.in_ground = false;
            projectile.despawn = 0;
            resting.leave(entity);
        }
        let delta = end - start;
        let margin = Projectile::margin(state.age);
        let stream = KeyedStream::new(&state.key, tick);
        match entity_hit(&world, &mut projectile, start, delta, margin) {
            Some(hit) => {
                let point = start + delta * hit.entry;
                let struck = Struck {
                    key: state.key,
                    candidate: hit.candidate,
                    velocity: delta,
                    tick,
                };
                let ends = strike(
                    &struck,
                    &mut projectile,
                    &targets,
                    &types,
                    &rules,
                    &mut pending,
                    &mut writes,
                );
                if projectile.kind == ProjectileKind::Arrow {
                    play(
                        SoundAt::arrow_hit(point, &stream),
                        sound_seed(&state.key, tick),
                        streams,
                        &mut changes,
                    );
                }
                if ends {
                    commands.entity(entity).despawn();
                    continue;
                }
                let velocity = delta * BOUNCE;
                writes.push(EntityWrite {
                    target: state.key,
                    writer: WriterKey::Entity { key: state.key },
                    sequence: 0,
                    effect: EntityEffect::Place {
                        position: point,
                        velocity,
                    },
                });
                projectile.speed = velocity.length();
                continue;
            }
            None if outcome.kernel.landed || outcome.kernel.stuck_in_block => {
                if projectile.kind == ProjectileKind::Snowball {
                    commands.entity(entity).despawn();
                    continue;
                }
                play(
                    SoundAt::arrow_hit(end, &stream),
                    sound_seed(&state.key, tick),
                    streams,
                    &mut changes,
                );
                projectile.in_ground = true;
                projectile.shake = Projectile::SHAKE;
                projectile.despawn = 0;
                projectile.critical = false;
                resting.rest(entity, end);
            }
            None => {}
        }
        projectile.speed = state.body.velocity.length();
    }
}

fn sound_seed(key: &EntityUuid, tick: u64) -> u64 {
    (key.as_u128() as u64) ^ tick.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn entity_hit(
    world: &FlightWorld<'_, '_, '_>,
    projectile: &mut Projectile,
    start: DVec3,
    delta: DVec3,
    margin: f64,
) -> Option<EntityHit> {
    let end = start + delta;
    let reach = DVec3::splat(margin + OWNER_REACH);
    let (min, max) = (start.min(end) - reach, start.max(end) + reach);
    let around = Aabb::new(start.min(end), start.max(end)).inflate(DVec3::splat(OWNER_REACH));
    let mut owner_near = false;
    let mut best: Option<EntityHit> = None;
    let mut consider = |candidate: Candidate| {
        if projectile.owner == Some(candidate.key) && !projectile.left_owner {
            if around.intersects(&candidate.bounds) {
                owner_near = true;
            }
            return;
        }
        let larger = candidate.bounds.inflate(DVec3::splat(margin));
        let Some(entry) = segment_meets(start, delta, larger.min, larger.max) else {
            return;
        };
        let rank = match projectile.kind {
            ProjectileKind::Snowball => entry,
            ProjectileKind::Arrow => start.distance_squared(candidate.centre),
        };
        if best.is_none_or(|best| rank < best.rank) {
            best = Some(EntityHit {
                rank,
                entry,
                candidate,
            });
        }
    };
    for row in world.grid.in_box(min, max) {
        if row.flags.contains(GridFlags::SPECTATOR) {
            continue;
        }
        consider(Candidate {
            key: row.key,
            entity: world.index.get(row.key),
            bounds: Aabb::of_body(row.position, row.half_width * 2.0, row.height),
            centre: row.position,
        });
    }
    let query = Aabb::new(min, max);
    for (entity, key, transform, mode, body) in world.players.iter() {
        if mode.0 == GameMode::Spectator {
            continue;
        }
        let dimensions = body.dimensions();
        let bounds = Aabb::of_body(transform.translation, dimensions.width, dimensions.height);
        if !query.intersects(&bounds) {
            continue;
        }
        consider(Candidate {
            key: *key,
            entity: Some(entity),
            bounds,
            centre: transform.translation,
        });
    }
    if !owner_near {
        projectile.left_owner = true;
    }
    best
}

struct Struck {
    key: EntityUuid,
    candidate: Candidate,
    velocity: DVec3,
    tick: u64,
}

fn strike(
    struck: &Struck,
    projectile: &mut Projectile,
    targets: &Targets<'_, '_>,
    types: &ProjectileDamageTypes,
    rules: &DamageTypeRules,
    pending: &mut PendingHits,
    writes: &mut EntityWrites,
) -> bool {
    let Some(target) = struck.candidate.entity else {
        return projectile.kind == ProjectileKind::Snowball;
    };
    let (damage_type, amount) = match projectile.kind {
        ProjectileKind::Arrow => (
            types.arrow,
            arrow_amount(projectile, &struck.key, struck.tick),
        ),
        ProjectileKind::Snowball => (types.thrown, SNOWBALL_AMOUNT),
    };
    let Some(damage_type) = damage_type else {
        tracing::debug!("the damage types of the projectiles are not baked; the hit is lost");
        return true;
    };
    let mut source = DamageSource::of(damage_type);
    source.cause = Cause {
        causing: Some(projectile.owner.unwrap_or(struck.key)),
        direct: Some(struck.key),
    };
    let hit = Hit::new(amount, source);
    let report = targets.get(target).ok().map(|(vitals, defence)| {
        preview(
            vitals,
            &defence.copied().unwrap_or_default(),
            rules,
            &HurtRules::default(),
            hit,
        )
    });
    if projectile.kind == ProjectileKind::Arrow
        && report.is_some_and(|report| report.outcome == HitOutcome::Rejected)
    {
        return false;
    }
    pending.push(DamageRequest::new(
        target,
        hit,
        WriterKey::Entity { key: struck.key },
        struck.tick,
        0,
    ));
    if report.is_some_and(|report| report.outcome == HitOutcome::Full) {
        writes.push(EntityWrite {
            target: struck.candidate.key,
            writer: WriterKey::Entity { key: struck.key },
            sequence: 0,
            effect: EntityEffect::Knockback {
                power: KNOCKBACK_POWER,
                direction: DVec3::new(-struck.velocity.x, 0.0, -struck.velocity.z),
            },
        });
    }
    true
}

fn arrow_amount(projectile: &Projectile, key: &EntityUuid, tick: u64) -> f32 {
    let scaled = (projectile.speed * projectile.base_damage).clamp(0.0, f64::from(i32::MAX));
    let mut amount = scaled.ceil() as i64;
    if projectile.critical {
        let bound = (amount / 2 + 1).clamp(1, i64::from(u32::MAX)) as u32;
        let extra = i64::from(KeyedStream::new(key, tick).below(bound.saturating_add(1)));
        amount = amount.saturating_add(extra).min(i64::from(i32::MAX));
    }
    amount as f32
}
