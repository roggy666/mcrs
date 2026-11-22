use bevy_app::App;
use bevy_app::FixedUpdate;
use bevy_app::Plugin;
use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::Add;
use bevy_ecs::lifecycle::Remove;
use bevy_ecs::message::MessageReader;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::With;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::schedule::SystemCondition;
use bevy_ecs::schedule::SystemSet;
use bevy_ecs::schedule::common_conditions::resource_exists;
use bevy_ecs::schedule::common_conditions::resource_exists_and_changed;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Local;
use bevy_ecs::system::ParamSet;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use bevy_ecs::system::Single;
use bevy_utils::Parallel;
use mcrs_engine_link::Placement;
use mcrs_minecraft_brain::AiDelay;
use mcrs_minecraft_brain::HurtBy;
use mcrs_minecraft_brain::ItemTables;
use mcrs_minecraft_brain::KindTable;
use mcrs_minecraft_brain::MobBrain;
use mcrs_minecraft_brain::StageConfig;
use mcrs_minecraft_brain::WorldSeed as StageSeed;
use mcrs_minecraft_brain_stage::Arrival;
use mcrs_minecraft_brain_stage::ProjectionWorld;
use mcrs_minecraft_brain_stage::default_threads;
use mcrs_minecraft_engine::column_store::StoredColumnLandSet;
use mcrs_minecraft_engine_damage::DamageSet;
use mcrs_minecraft_engine_damage::Damaged;
use mcrs_minecraft_engine_damage::PendingHits;
use mcrs_minecraft_engine_entity_motion::MotionId;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_engine_physics::MobSpawnRequest;
use mcrs_minecraft_engine_physics::PhysicsTables;
use mcrs_minecraft_engine_physics::PlayerIndex;
use mcrs_minecraft_engine_world::block_update::BlockPlaced;
use mcrs_minecraft_engine_world::tick::TickSet;
use mcrs_minecraft_engine_world::world::dimension::Dimension;
use mcrs_minecraft_engine_world::world::lifecycle::stage::SectionStageChanged;
use mcrs_minecraft_engine_world::world::storage::view::WorldView;
use mcrs_minecraft_engine_world::world_seed::WorldSeed;
use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity::damage_type::DamageType;
use mcrs_minecraft_entity::damage_type_keys::MOB_ATTACK;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_registries::CurrentRegistries;
use mcrs_minecraft_registries::Registries;
use mcrs_minecraft_resource::ResourceLocation;
use mcrs_minecraft_resource::TagKey;
use mcrs_minecraft_resource::rl;
use mcrs_minecraft_volume::Volume;

use crate::BlockGoalWorld;
use crate::BrainState;
use crate::MobAi;
use crate::MobHit;
use crate::MobNavigation;
use crate::RangedAttack;
use crate::ai_window::AiWindow;
use crate::mob_ai::BrainOut;
use crate::mob_ai::Hitting;
use crate::mob_ai::Thinking;
use crate::panic_causes_in::panic_causes_in;
use crate::projection_feed::feed_projection;
use crate::recompute::flag_recomputes;

const DAMAGE_TYPE: ResourceLocation<'static> = rl!("minecraft:damage_type");

#[derive(Clone, Copy, Debug)]
pub struct MobBrainPlugin<B> {
    pub max_lag: u8,
    pub delay: AiDelay,
    pub placement: Placement,
    pub threads: usize,
    pub brain_for: fn(&'static EntityTypeData) -> Option<B>,
    pub panic_causes: TagKey<'static, DamageType>,
    pub item_tables: fn(&Registries) -> ItemTables,
    pub kinds: fn() -> KindTable,
}

impl<B> MobBrainPlugin<B> {
    #[must_use]
    pub fn new(
        brain_for: fn(&'static EntityTypeData) -> Option<B>,
        panic_causes: TagKey<'static, DamageType>,
        item_tables: fn(&Registries) -> ItemTables,
        kinds: fn() -> KindTable,
    ) -> Self {
        Self {
            max_lag: StageConfig::DEFAULT_MAX_LAG,
            delay: AiDelay::AtLeast(1),
            placement: Placement::Thread,
            threads: default_threads(),
            brain_for,
            panic_causes,
            item_tables,
            kinds,
        }
    }

    #[must_use]
    pub const fn with_placement(mut self, placement: Placement, threads: usize) -> Self {
        self.placement = placement;
        self.threads = threads;
        self
    }

    #[must_use]
    pub const fn with_delay(mut self, max_lag: u8, delay: AiDelay) -> Self {
        self.max_lag = max_lag;
        self.delay = delay;
        self
    }
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MobBrainSet {
    Prepare,
}

impl<B> Plugin for MobBrainPlugin<B>
where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        let ready = resource_exists::<PhysicsTables>.and_then(resource_exists::<MobAi<B>>);
        app.add_message::<MobHit>();
        app.add_message::<RangedAttack>();
        app.add_message::<SectionStageChanged>();
        app.add_message::<BlockPlaced>();
        app.configure_sets(
            FixedUpdate,
            MobBrainSet::Prepare
                .after(TickSet::ClientInput)
                .after(StoredColumnLandSet)
                .before(TickSet::Simulate)
                .run_if(ready),
        );
        app.add_observer(admit_brain::<B>);
        app.add_observer(release_brain::<B>);
        app.add_systems(
            FixedUpdate,
            (
                set_tables::<B>(self.panic_causes, self.item_tables)
                    .run_if(
                        resource_exists::<MobAi<B>>
                            .and_then(resource_exists::<PhysicsTables>)
                            .and_then(resource_exists_and_changed::<CurrentRegistries>),
                    )
                    .after(TickSet::ClientInput)
                    .before(MobBrainSet::Prepare),
                feed_projection::<B>
                    .in_set(MobBrainSet::Prepare)
                    .before(prepare::<B>),
                flag_recomputes
                    .in_set(MobBrainSet::Prepare)
                    .before(prepare::<B>),
                prepare::<B>.in_set(MobBrainSet::Prepare),
                record_hurt_by::<B>
                    .in_set(DamageSet::Report)
                    .run_if(resource_exists::<MobAi<B>>.and_then(resource_exists::<PendingHits>)),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        let seed = StageSeed(
            app.world()
                .get_resource::<WorldSeed>()
                .map_or(0, |seed| seed.0),
        );
        let config = match StageConfig::new(seed).with_delay(self.max_lag, self.delay) {
            Ok(config) => config,
            Err(error) => {
                tracing::error!(%error, "the AI keeps its default delay");
                StageConfig::new(seed)
            }
        };
        app.insert_resource(MobAi::new(
            config,
            self.brain_for,
            (self.kinds)(),
            self.placement,
            self.threads,
        ));
    }
}

fn admit_brain<B>(
    add: On<Add<Mob>>,
    mobs: Query<(&Mob, &MotionId, &MotionState)>,
    ai: Option<ResMut<MobAi<B>>>,
    mut commands: Commands,
) where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    let entity = add.event().entity;
    let (Some(mut ai), Ok((mob, id, state))) = (ai, mobs.get(entity)) else {
        return;
    };
    if mob.looks.no_ai {
        return;
    }
    let Some(brain) = ai.brain_for(mob.kind) else {
        return;
    };
    let rules = MobBrain::<ProjectionWorld<'_>>::rules(&brain);
    let Some(brain_state) = BrainState::new(mob.kind, rules, mob.baby) else {
        return;
    };
    ai.admit(Arrival {
        id: id.0,
        key: state.0.key,
        tile: TileKey::at(state.0.body.position),
        phase: state.0.phase,
        brain,
    });
    commands
        .entity(entity)
        .insert((brain_state, MobNavigation::default()));
}

fn release_brain<B>(
    remove: On<Remove<BrainState>>,
    ids: Query<&MotionId>,
    ai: Option<ResMut<MobAi<B>>>,
) where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    if let (Some(mut ai), Ok(id)) = (ai, ids.get(remove.event().entity)) {
        ai.release(id.0);
    }
}

fn set_tables<B>(
    panic_causes: TagKey<'static, DamageType>,
    item_tables: fn(&Registries) -> ItemTables,
) -> impl FnMut(
    Single<Entity, With<Dimension>>,
    WorldView,
    Res<CurrentRegistries>,
    Res<PhysicsTables>,
    ResMut<MobAi<B>>,
)
where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    move |dimension, world, registries, physics, mut ai| {
        let mob_attack = registries
            .names_of(DAMAGE_TYPE)
            .and_then(|names| names.iter().position(|name| *name == MOB_ATTACK.location()))
            .and_then(|index| u32::try_from(index).ok())
            .unwrap_or(0);
        let height = world
            .get(*dimension)
            .map_or(BuildHeight { min: 0, max: 0 }, |blocks| {
                let bounds = blocks.bounds();
                BuildHeight {
                    min: bounds.min.y,
                    max: bounds.max.y,
                }
            });
        ai.configure(
            physics.tables(),
            item_tables(&registries),
            panic_causes_in(&registries, panic_causes),
            mob_attack,
            height,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each resource, query and writer is its own parameter so the scheduler sees the access"
)]
fn prepare<B>(
    dimension: Single<Entity, With<Dimension>>,
    world: WorldView,
    tables: Res<PhysicsTables>,
    players: Res<PlayerIndex>,
    mut mobs: ParamSet<(Thinking, Hitting)>,
    mut pending: Option<ResMut<PendingHits>>,
    mut ai: ResMut<MobAi<B>>,
    mut threads: Local<Parallel<BrainOut>>,
    mut window: Local<AiWindow>,
    mut hits: MessageWriter<MobHit>,
    mut shots: MessageWriter<RangedAttack>,
    mut spawns: MessageWriter<MobSpawnRequest>,
) where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    let Some(blocks) = world.get(*dimension) else {
        return;
    };
    let goal_world = BlockGoalWorld::new(&blocks, tables.tables());
    ai.prepare(
        &goal_world,
        players.table().clone(),
        players.entities(),
        &mut mobs,
        pending.as_deref_mut(),
        &mut threads,
    );
    window.add(ai.tick(), ai.stats());
    hits.write_batch(ai.take_player_hits());
    shots.write_batch(ai.take_shots());
    spawns.write_batch(ai.take_spawns());
}

fn record_hurt_by<B>(
    mut damaged: MessageReader<Damaged>,
    ai: Res<MobAi<B>>,
    mut brains: Query<&mut BrainState>,
) where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    let tick = ai.tick() as u32;
    for hit in damaged.read() {
        let Some(attacker) = ai.attacker_of(hit.target) else {
            continue;
        };
        if let Ok(mut brain) = brains.get_mut(hit.target) {
            brain.hurt_by = HurtBy::by(attacker, tick);
            brain.love = 0;
        }
    }
}
