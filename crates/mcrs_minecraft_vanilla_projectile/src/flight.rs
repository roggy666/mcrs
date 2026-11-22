use bevy_ecs::entity::Entity;
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
use mcrs_minecraft_engine_damage::DamageRequest;
use mcrs_minecraft_engine_damage::PendingHits;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_entity_motion::StepOutcome;
use mcrs_minecraft_engine_physics::MobIndex;
use mcrs_minecraft_engine_world::birth_clock::BirthClock;
use mcrs_minecraft_entity::damage_type_keys::ARROW;
use mcrs_minecraft_entity::damage_type_keys::THROWN;
use mcrs_minecraft_entity_key::Cause;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::KeyedStream;
use mcrs_minecraft_entity_key::WriterKey;
use mcrs_minecraft_physics::Aabb;
use mcrs_minecraft_physics::BodyGrid;
use mcrs_minecraft_physics::GridFlags;
use mcrs_minecraft_physics::GridRow;
use mcrs_minecraft_physics::segment_meets;
use mcrs_minecraft_registries::CurrentRegistries;
use mcrs_minecraft_resource::ResourceLocation;
use mcrs_minecraft_resource::rl;

use crate::Projectile;
use crate::ProjectileKind;

const DAMAGE_TYPE: ResourceLocation<'static> = rl!("minecraft:damage_type");
const BOUNCE: f64 = -0.1;
const OWNER_REACH: f64 = 1.0;
const SNOWBALL_AMOUNT: f32 = 0.0;

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

#[derive(Clone, Copy, Debug)]
struct EntityHit {
    rank: f64,
    entry: f64,
    row: GridRow,
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each resource, query and writer is its own parameter"
)]
pub(crate) fn fly_projectiles(
    mut projectiles: Flying,
    targets: Targets,
    grid: Res<BodyGrid>,
    index: Res<MobIndex>,
    clock: Res<BirthClock>,
    types: Res<ProjectileDamageTypes>,
    rules: Res<DamageTypeRules>,
    mut pending: ResMut<PendingHits>,
    mut commands: Commands,
) {
    let tick = clock.0;
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
        }
        let delta = end - start;
        let margin = Projectile::margin(state.age);
        let hit = entity_hit(&grid, &mut projectile, start, delta, margin);
        match hit {
            Some(hit) => {
                let point = start + delta * hit.entry;
                let struck = Struck {
                    key: state.key,
                    target: &hit.row,
                    tick,
                };
                let ends = strike(
                    &struck,
                    &mut projectile,
                    &targets,
                    &index,
                    &types,
                    &rules,
                    &mut pending,
                );
                if ends {
                    commands.entity(entity).despawn();
                    continue;
                }
                let mut bounced = motion.0;
                bounced.body.position = point;
                bounced.body.velocity = delta * BOUNCE;
                commands.entity(entity).insert(MotionState(bounced));
                projectile.speed = bounced.body.velocity.length();
                continue;
            }
            None if outcome.kernel.landed || outcome.kernel.stuck_in_block => {
                if projectile.kind == ProjectileKind::Snowball {
                    commands.entity(entity).despawn();
                    continue;
                }
                projectile.in_ground = true;
                projectile.shake = Projectile::SHAKE;
                projectile.despawn = 0;
            }
            None => {}
        }
        projectile.speed = motion.0.body.velocity.length();
    }
}

fn entity_hit(
    grid: &BodyGrid,
    projectile: &mut Projectile,
    start: DVec3,
    delta: DVec3,
    margin: f64,
) -> Option<EntityHit> {
    let end = start + delta;
    let reach = DVec3::splat(margin + OWNER_REACH);
    let (min, max) = (start.min(end) - reach, start.max(end) + reach);
    let mut owner_near = false;
    let mut best: Option<EntityHit> = None;
    for row in grid.in_box(min, max) {
        if row.flags.contains(GridFlags::SPECTATOR) {
            continue;
        }
        let bounds = Aabb::of_body(row.position, row.half_width * 2.0, row.height);
        if projectile.owner == Some(row.key) && !projectile.left_owner {
            let around =
                Aabb::new(start.min(end), start.max(end)).inflate(DVec3::splat(OWNER_REACH));
            if around.intersects(&bounds) {
                owner_near = true;
            }
            continue;
        }
        let larger = bounds.inflate(DVec3::splat(margin));
        let Some(entry) = segment_meets(start, delta, larger.min, larger.max) else {
            continue;
        };
        let rank = match projectile.kind {
            ProjectileKind::Snowball => entry,
            ProjectileKind::Arrow => start.distance_squared(row.position),
        };
        if best.is_none_or(|best| rank < best.rank) {
            best = Some(EntityHit {
                rank,
                entry,
                row: *row,
            });
        }
    }
    if !owner_near {
        projectile.left_owner = true;
    }
    best
}

struct Struck<'a> {
    key: EntityUuid,
    target: &'a GridRow,
    tick: u64,
}

fn strike(
    struck: &Struck<'_>,
    projectile: &mut Projectile,
    targets: &Targets<'_, '_>,
    index: &MobIndex,
    types: &ProjectileDamageTypes,
    rules: &DamageTypeRules,
    pending: &mut PendingHits,
) -> bool {
    let Some(target) = index.get(struck.target.key) else {
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
    if projectile.kind == ProjectileKind::Arrow
        && let Ok((vitals, defence)) = targets.get(target)
    {
        let report = preview(
            vitals,
            &defence.copied().unwrap_or_default(),
            rules,
            &HurtRules::default(),
            hit,
        );
        if report.outcome == HitOutcome::Rejected {
            return false;
        }
    }
    pending.push(DamageRequest::new(
        target,
        hit,
        WriterKey::Entity { key: struck.key },
        struck.tick,
        0,
    ));
    true
}

fn arrow_amount(projectile: &Projectile, key: &EntityUuid, tick: u64) -> f32 {
    let scaled = (projectile.speed * projectile.base_damage).clamp(0.0, f64::from(i32::MAX));
    let mut amount = scaled.ceil() as i64;
    if projectile.critical {
        let bound = amount / 2 + 1;
        let extra = i64::from(
            KeyedStream::new(key, tick).below(bound.clamp(1, i64::from(u32::MAX)) as u32 + 1),
        );
        amount = amount.saturating_add(extra).min(i64::from(i32::MAX));
    }
    amount as f32
}
