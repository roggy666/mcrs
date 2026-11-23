use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::RunSystemOnce;
use bevy_ecs::system::Single;
use bevy_tasks::ComputeTaskPool;
use bevy_tasks::TaskPoolBuilder;
use glam::DVec3;
use mcrs_engine_link::Placement;
use mcrs_minecraft_brain::AiDelay;
use mcrs_minecraft_brain::ItemTables;
use mcrs_minecraft_brain::PanicCauses;
use mcrs_minecraft_brain::StageConfig;
use mcrs_minecraft_brain::WorldSeed;
use mcrs_minecraft_damage::DamageSource;
use mcrs_minecraft_damage::Hit;
use mcrs_minecraft_damage::HitRecord;
use mcrs_minecraft_engine_brain::MobAi;
use mcrs_minecraft_engine_brain::MobBrainPlugin;
use mcrs_minecraft_engine_damage::DamageRequest;
use mcrs_minecraft_engine_damage::PendingHits;
use mcrs_minecraft_engine_entity_motion::MobIntent;
use mcrs_minecraft_engine_entity_motion::MotionId;
use mcrs_minecraft_engine_entity_motion::MotionKind;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_physics::FLOOR;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_engine_physics::PhysicsTables;
use mcrs_minecraft_engine_physics::TestScene;
use mcrs_minecraft_engine_physics::mob_body;
use mcrs_minecraft_engine_physics::mob_row;
use mcrs_minecraft_engine_physics::mob_vitals;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::session::Owner;
use mcrs_minecraft_engine_world::session::PlayerSession;
use mcrs_minecraft_engine_world::world::dimension::Dimension;
use mcrs_minecraft_engine_world::world::storage::view::WorldView;
use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity::damage_type_tag_keys::PANIC_CAUSES;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::WriterKey;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_pose::Pose;
use mcrs_minecraft_protocol_game::entity::MobLooks;
use mcrs_minecraft_vanilla_mobs::VanillaBrain;
use mcrs_minecraft_vanilla_mobs::item_tables;
use mcrs_minecraft_vanilla_mobs::kind_table;
use mcrs_minecraft_vanilla_mobs::vanilla_brain;
use mcrs_minecraft_volume::Volume;

const PIGS: u32 = 2000;
const TICKS: u64 = 200;
const SHORT_TICKS: u64 = 60;
const HALF_TILES: i32 = 3;
const CACTUS: u32 = 7;
const HIT_TICK: u64 = 50;
static THREADS: AtomicUsize = AtomicUsize::new(1);
const SHORT_GOLDEN: u64 = 15_051_530_398_365_880_483;
const LONG_GOLDEN: u64 = 7_854_557_506_117_327_600;

fn positions() -> Vec<DVec3> {
    let edge = f64::from(HALF_TILES * 32 - 2);
    let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
    let mut unit = || {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    (0..PIGS)
        .map(|_| {
            let x = (unit() * 2.0 - 1.0) * edge;
            let z = (unit() * 2.0 - 1.0) * edge;
            DVec3::new(x, f64::from(FLOOR), z)
        })
        .collect()
}

fn use_threads(threads: usize) {
    THREADS.store(threads, Ordering::Relaxed);
    ComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(threads).build());
    assert_eq!(
        ComputeTaskPool::get().thread_num(),
        threads,
        "each test runs in its own process, so it sets the size of the pool"
    );
}

struct Run {
    scene: TestScene,
    struck: Vec<Entity>,
    host_only: bool,
}

#[derive(Clone, Copy, Debug)]
struct Setup {
    placement: Placement,
    delay: AiDelay,
    host_only: bool,
    slow: Duration,
    tile_split: usize,
}

const INLINE: Setup = Setup {
    placement: Placement::Inline,
    delay: AiDelay::AtLeast(1),
    host_only: false,
    slow: Duration::ZERO,
    tile_split: StageConfig::DEFAULT_TILE_SPLIT,
};

const SLOW_STAGE: Duration = Duration::from_millis(80);

fn scene_height(
    dimension: Single<Entity, With<Dimension>>,
    view: WorldView,
) -> Option<BuildHeight> {
    let bounds = view.get(*dimension)?.bounds();
    Some(BuildHeight {
        min: bounds.min.y,
        max: bounds.max.y,
    })
}

impl Run {
    fn new(order: &[u32], setup: Setup) -> Self {
        let mut scene = TestScene::new(HALF_TILES, 0, 0, 1);
        scene.app.add_plugins(MobBrainPlugin::new(
            vanilla_brain,
            PANIC_CAUSES,
            item_tables,
            kind_table,
        ));
        let config = StageConfig::new(WorldSeed(42))
            .with_tile_split(setup.tile_split)
            .with_delay(StageConfig::DEFAULT_MAX_LAG, setup.delay)
            .unwrap_or_else(|_| StageConfig::new(WorldSeed(42)));
        let mut ai = MobAi::new(
            config,
            vanilla_brain,
            kind_table(),
            setup.placement,
            THREADS.load(Ordering::Relaxed),
        );
        let world = scene.app.world_mut();
        let physics = world.resource::<PhysicsTables>().tables().clone();
        let height = world
            .run_system_once(scene_height)
            .ok()
            .flatten()
            .unwrap_or(BuildHeight { min: -64, max: 319 });
        ai.configure(
            &physics,
            ItemTables::default(),
            PanicCauses::new([CACTUS]),
            0,
            height,
        );
        if !setup.slow.is_zero() {
            ai.slow_stage(setup.slow);
        }
        world.insert_resource(ai);
        world.spawn((
            Player,
            Owner(PlayerSession(1)),
            EntityUuid::from_u128(u128::MAX),
            Transform {
                translation: DVec3::new(0.5, f64::from(FLOOR), 0.5),
                ..Transform::IDENTITY
            },
            Pose::Standing,
        ));
        let positions = positions();
        let mut struck = Vec::new();
        for &index in order {
            let Some(&position) = positions.get(index as usize) else {
                continue;
            };
            let row = mob_row(&EntityTypeData::PIG);
            let key = EntityUuid::from_u128(u128::from(index));
            let entity = world
                .spawn((
                    MotionState(mob_body(&EntityTypeData::PIG, &row, key, position)),
                    MotionKind(row),
                    MotionId(EntityId(u64::from(index))),
                    MobIntent::default(),
                    mob_vitals(&EntityTypeData::PIG),
                    HitRecord::default(),
                    Mob {
                        kind: &EntityTypeData::PIG,
                        uuid: u128::from(index),
                        looks: MobLooks::default(),
                        custom_name: None,
                        tags: Vec::new(),
                        baby: false,
                    },
                ))
                .id();
            if index < 40 {
                struck.push(entity);
            }
        }
        Self {
            scene,
            struck,
            host_only: setup.host_only,
        }
    }

    fn tick(&mut self, tick: u64) -> u64 {
        if tick + 1 == HIT_TICK {
            let mut pending = self.scene.app.world_mut().resource_mut::<PendingHits>();
            for &entity in &self.struck {
                pending.push(DamageRequest::new(
                    entity,
                    Hit::new(1.0, DamageSource::of(CACTUS)),
                    WriterKey::Command { sequence: 0 },
                    HIT_TICK,
                    0,
                ));
            }
        }
        self.scene.step();
        self.scene
            .app
            .world_mut()
            .resource_mut::<MobAi<VanillaBrain>>()
            .settle();
        self.hash()
    }

    fn mobs(&mut self) -> Vec<(EntityId, MotionState)> {
        let world = self.scene.app.world_mut();
        let mut query = world.query::<(&MotionId, &MotionState)>();
        let mut mobs: Vec<(EntityId, MotionState)> = query
            .iter(world)
            .filter(|(_, state)| state.0.living.is_some())
            .map(|(id, state)| (id.0, *state))
            .collect();
        mobs.sort_by_key(|(_, state)| state.0.key);
        mobs
    }

    fn hash(&mut self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for (id, state) in self.mobs() {
            id.hash(&mut hasher);
            let state = state.0;
            state
                .body
                .position
                .to_array()
                .map(f64::to_bits)
                .hash(&mut hasher);
            if let Some(living) = state.living {
                [
                    living.orientation.yaw,
                    living.orientation.pitch,
                    living.orientation.head_yaw,
                    living.orientation.body_yaw,
                ]
                .map(f32::to_bits)
                .hash(&mut hasher);
            }
        }
        if !self.host_only
            && let Some(stage) = self.ai().inline_stage()
        {
            for (tile, brains) in stage.stage().tiles() {
                tile.hash(&mut hasher);
                for entry in brains.iter() {
                    entry.mob.hash(&mut hasher);
                    entry.wake.hash(&mut hasher);
                    entry.brain.hash(&mut hasher);
                }
            }
        }
        hasher.finish()
    }

    fn ai(&self) -> &MobAi<VanillaBrain> {
        self.scene.app.world().resource::<MobAi<VanillaBrain>>()
    }

    fn positions(&mut self) -> Vec<DVec3> {
        self.mobs()
            .into_iter()
            .map(|(_, state)| state.0.body.position)
            .collect()
    }
}

struct Crowd {
    hash: u64,
    walked: usize,
    evaluations: u64,
    waits: u64,
    lag_max: u64,
}

fn forward() -> Vec<u32> {
    (0..PIGS).collect()
}

#[expect(
    clippy::indexing_slicing,
    reason = "a test helper reports a failure by panicking"
)]
fn run_orders(ticks: u64, orders: &[Vec<u32>], setup: Setup) -> Crowd {
    let mut runs: Vec<Run> = orders.iter().map(|order| Run::new(order, setup)).collect();
    let start = runs[0].positions();
    let mut hash = 0;
    for tick in 1..=ticks {
        let hashes: Vec<u64> = runs.iter_mut().map(|run| run.tick(tick)).collect();
        assert!(
            hashes.iter().all(|other| *other == hashes[0]),
            "tick {tick}: the hashes of the spawn orders differ: {hashes:?}"
        );
        hash = hashes[0];
    }
    let end = runs[0].positions();
    let walked = start
        .iter()
        .zip(&end)
        .filter(|(from, to)| from.with_y(0.0).distance(to.with_y(0.0)) > 1.0)
        .count();
    let stats = *runs[0].ai().stats();
    Crowd {
        hash,
        walked,
        evaluations: stats.evaluations_total,
        waits: stats.waits,
        lag_max: stats.lag_max_seen,
    }
}

fn assert_short_crowd(crowd: &Crowd) {
    assert_eq!(
        crowd.hash, SHORT_GOLDEN,
        "every thread count and spawn order ends on the same state"
    );
    assert!(
        crowd.walked > 0,
        "the pigs near the player and the hit pigs walk: {}",
        crowd.walked
    );
    assert!(
        crowd.evaluations > u64::from(PIGS),
        "every pig is evaluated: {}",
        crowd.evaluations
    );
}

#[test]
fn the_ai_of_a_crowd_ends_on_the_golden_state_at_1_thread() {
    use_threads(1);
    assert_short_crowd(&run_orders(SHORT_TICKS, &[forward()], INLINE));
}

#[test]
fn the_ai_of_a_crowd_ends_on_the_golden_state_at_8_threads_in_any_spawn_order() {
    use_threads(8);
    let reversed: Vec<u32> = (0..PIGS).rev().collect();
    let mut interleaved: Vec<u32> = (0..PIGS).filter(|index| index % 2 == 1).collect();
    interleaved.extend((0..PIGS).filter(|index| index % 2 == 0));
    assert_short_crowd(&run_orders(
        SHORT_TICKS,
        &[forward(), reversed, interleaved],
        INLINE,
    ));
}

#[test]
fn the_goldens_hold_when_every_tile_is_split_to_one_brain() {
    let split = Setup {
        tile_split: 1,
        ..INLINE
    };
    use_threads(8);
    assert_short_crowd(&run_orders(SHORT_TICKS, &[forward()], split));
}

#[test]
fn a_stage_on_its_own_thread_with_a_fixed_delay_ends_on_the_inline_state() {
    use_threads(4);
    let fixed = AiDelay::Fixed(1);
    let inline = run_orders(
        SHORT_TICKS,
        &[forward()],
        Setup {
            placement: Placement::Inline,
            delay: fixed,
            host_only: true,
            slow: Duration::ZERO,
            tile_split: StageConfig::DEFAULT_TILE_SPLIT,
        },
    );
    let threaded = run_orders(
        SHORT_TICKS,
        &[forward()],
        Setup {
            placement: Placement::Thread,
            delay: fixed,
            host_only: true,
            slow: Duration::ZERO,
            tile_split: StageConfig::DEFAULT_TILE_SPLIT,
        },
    );
    assert_eq!(
        inline.hash, threaded.hash,
        "the placement of the stage does not change the bodies and the brain states"
    );
    assert!(
        threaded.walked > 0,
        "the hit pigs walk: {}",
        threaded.walked
    );
}

#[test]
fn a_stage_on_its_own_thread_with_the_default_delay_keeps_every_decision_in_the_lag() {
    use_threads(4);
    let threaded = run_orders(
        SHORT_TICKS,
        &[forward()],
        Setup {
            placement: Placement::Thread,
            delay: AiDelay::AtLeast(1),
            host_only: true,
            slow: Duration::ZERO,
            tile_split: StageConfig::DEFAULT_TILE_SPLIT,
        },
    );
    assert!(
        threaded.walked > 0,
        "the hit pigs walk: {}",
        threaded.walked
    );
    assert!(
        threaded.evaluations > u64::from(PIGS),
        "every pig is evaluated: {}",
        threaded.evaluations
    );
}

#[test]
fn a_slow_stage_waits_only_at_the_lag_and_ends_on_the_state_of_the_fixed_delay() {
    use_threads(4);
    let fixed = run_orders(
        SHORT_TICKS,
        &[forward()],
        Setup {
            placement: Placement::Inline,
            delay: AiDelay::Fixed(StageConfig::DEFAULT_MAX_LAG),
            host_only: true,
            slow: Duration::ZERO,
            tile_split: StageConfig::DEFAULT_TILE_SPLIT,
        },
    );
    let slow = run_orders(
        SHORT_TICKS,
        &[forward()],
        Setup {
            placement: Placement::Thread,
            delay: AiDelay::AtLeast(1),
            host_only: true,
            slow: SLOW_STAGE,
            tile_split: StageConfig::DEFAULT_TILE_SPLIT,
        },
    );
    assert_eq!(
        slow.hash, fixed.hash,
        "a stage that is always late applies every decision at the lag, as the fixed delay does"
    );
    assert!(slow.waits > 0, "the tick waited for the late stage");
    assert_eq!(
        slow.lag_max,
        u64::from(StageConfig::DEFAULT_MAX_LAG),
        "no decision is older than the lag"
    );
    assert_eq!(fixed.lag_max, u64::from(StageConfig::DEFAULT_MAX_LAG));
}

mod exhaustive {
    use super::*;

    fn assert_long_crowd(threads: usize) {
        use_threads(threads);
        let crowd = run_orders(TICKS, &[forward()], INLINE);
        assert_eq!(
            crowd.hash, LONG_GOLDEN,
            "every thread count ends on the same state"
        );
        assert!(
            crowd.walked > 20,
            "the pigs near the player and the hit pigs walk: {}",
            crowd.walked
        );
        assert!(
            crowd.evaluations > u64::from(PIGS),
            "every pig is evaluated: {}",
            crowd.evaluations
        );
    }

    #[test]
    fn the_ai_of_a_crowd_ends_on_the_golden_state_at_1_thread_in_a_long_run() {
        assert_long_crowd(1);
    }

    #[test]
    fn the_ai_of_a_crowd_ends_on_the_golden_state_at_8_threads_in_a_long_run() {
        assert_long_crowd(8);
    }

    #[test]
    fn the_ai_of_a_crowd_ends_on_the_golden_state_at_16_threads_in_a_long_run() {
        assert_long_crowd(16);
    }
}
