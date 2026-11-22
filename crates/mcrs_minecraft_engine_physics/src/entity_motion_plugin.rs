use bevy_app::App;
use bevy_app::FixedUpdate;
use bevy_app::Plugin;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageReader;
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::schedule::common_conditions::resource_exists;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use bevy_ecs::system::Single;
use bevy_ecs::system::SystemParam;
use mcrs_minecraft_collision_table::BlockWorld;
use mcrs_minecraft_engine_entity_motion::Crowding;
use mcrs_minecraft_engine_entity_motion::MotionPass;
use mcrs_minecraft_engine_entity_motion::MotionPassPlugin;
use mcrs_minecraft_engine_entity_motion::MotionStep;
use mcrs_minecraft_engine_entity_motion::MotionSteps;
use mcrs_minecraft_engine_entity_motion::StirredTiles;
use mcrs_minecraft_engine_world::birth_clock::BirthClock;
use mcrs_minecraft_engine_world::game_rules::GameRules;
use mcrs_minecraft_engine_world::tick::TickSet;
use mcrs_minecraft_engine_world::voxel_update::BlockPlaced;
use mcrs_minecraft_engine_world::world::dimension::Dimension;
use mcrs_minecraft_engine_world::world::storage::view::WorldView;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_minecraft_game_rule::GameRule;
use mcrs_minecraft_physics::PushRules;
use mcrs_minecraft_physics::Side;

use crate::PhysicsTables;
use crate::crowd_damage_type::CrowdDamageType;

#[derive(Clone, Copy, Debug, Default)]
pub struct EntityMotionPlugin;

impl Plugin for EntityMotionPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MotionPassPlugin>() {
            app.add_plugins(MotionPassPlugin);
        }
        app.init_resource::<BirthClock>();
        app.add_message::<BlockPlaced>();
        app.configure_sets(FixedUpdate, MotionPass.in_set(TickSet::Simulate));
        app.add_systems(FixedUpdate, stir_tiles.in_set(TickSet::DomainInput));
        app.add_systems(
            FixedUpdate,
            step_motion
                .run_if(resource_exists::<PhysicsTables>)
                .in_set(MotionSteps),
        );
    }
}

fn stir_tiles(mut placed: MessageReader<BlockPlaced>, mut stirred: ResMut<StirredTiles>) {
    let tiles = placed
        .read()
        .map(|block| TileKey::at(block.block_pos.as_dvec3()));
    stirred.replace(tiles);
}

#[derive(SystemParam)]
struct CrowdSettings<'w> {
    damage_type: Option<Res<'w, CrowdDamageType>>,
    rules: Option<Res<'w, GameRules>>,
    cap: Option<Res<'w, CollisionCap>>,
}

impl CrowdSettings<'_> {
    fn crowding(&self) -> Crowding {
        Crowding {
            rules: push_rules(self.rules.as_deref(), self.cap.as_deref()),
            damage_type: self.damage_type.as_deref().and_then(|crowd| crowd.0),
        }
    }
}

fn step_motion(
    dimension: Single<Entity, With<Dimension>>,
    world: WorldView,
    tables: Res<PhysicsTables>,
    clock: Res<BirthClock>,
    crowd: CrowdSettings,
    mut step: MotionStep,
) {
    let Some(blocks) = world.get(*dimension) else {
        return;
    };
    let world = BlockWorld {
        blocks: &blocks,
        table: tables.collision(),
    };
    step.run(
        &world,
        tables.tables(),
        Side::Server { authority: true },
        clock.0,
        &crowd.crowding(),
    );
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollisionCap(pub u16);

impl Default for CollisionCap {
    fn default() -> Self {
        Self(PushRules::DEFAULT_CAP as u16)
    }
}

pub(crate) fn push_rules(rules: Option<&GameRules>, cap: Option<&CollisionCap>) -> PushRules {
    PushRules {
        max_entity_cramming: rules.map_or(PushRules::default().max_entity_cramming, |rules| {
            rules.get(GameRule::MaxEntityCramming)
        }),
        cap: usize::from(cap.copied().unwrap_or_default().0),
    }
}
