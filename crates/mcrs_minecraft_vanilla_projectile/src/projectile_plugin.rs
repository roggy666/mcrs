use bevy_app::App;
use bevy_app::FixedUpdate;
use bevy_app::Plugin;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::schedule::SystemSet;
use bevy_ecs::schedule::common_conditions::any_with_component;
use bevy_ecs::schedule::common_conditions::on_message;
use bevy_ecs::schedule::common_conditions::resource_exists_and_changed;
use mcrs_minecraft_engine::tile_events::TileEvents;
use mcrs_minecraft_engine::use_item_on_request::BlockUseSet;
use mcrs_minecraft_engine::use_item_request::ReleaseItemRequest;
use mcrs_minecraft_engine::use_item_request::UseItemRequest;
use mcrs_minecraft_engine_brain::MobBrainSet;
use mcrs_minecraft_engine_brain::RangedAttack;
use mcrs_minecraft_engine_damage::DamageSet;
use mcrs_minecraft_engine_damage::Damaged;
use mcrs_minecraft_engine_entity_motion::MotionPass;
use mcrs_minecraft_engine_physics::MobPublish;
use mcrs_minecraft_engine_world::tick::TickSet;
use mcrs_minecraft_registries::CurrentRegistries;

use crate::Drawing;
use crate::Projectile;
use crate::ProjectileLaunch;
use crate::bow::count_draws;
use crate::bow::release_item_use;
use crate::bow::start_item_use;
use crate::flight::ProjectileDamageTypes;
use crate::flight::bake_damage_types;
use crate::flight::fly_projectiles;
use crate::launch::aim_ranged_attacks;
use crate::launch::forget_projectile;
use crate::launch::launch_projectiles;
use crate::pickup::RestingArrows;
use crate::pickup::forget_resting;
use crate::pickup::has_resting_arrows;
use crate::pickup::pick_up_arrows;
use crate::player_hits::give_player_vitals;
use crate::player_hits::report_player_hits;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProjectilePlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectileSet;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectileDamageTypes>()
            .init_resource::<RestingArrows>()
            .init_resource::<TileEvents>()
            .add_message::<ProjectileLaunch>()
            .add_message::<RangedAttack>()
            .add_message::<UseItemRequest>()
            .add_message::<ReleaseItemRequest>()
            .add_message::<Damaged>()
            .add_observer(forget_projectile)
            .add_observer(forget_resting)
            .add_observer(give_player_vitals)
            .configure_sets(
                FixedUpdate,
                ProjectileSet
                    .in_set(TickSet::Simulate)
                    .after(MotionPass)
                    .before(DamageSet::Collect),
            )
            .add_systems(
                FixedUpdate,
                bake_damage_types
                    .run_if(resource_exists_and_changed::<CurrentRegistries>)
                    .in_set(TickSet::DomainInput),
            )
            .add_systems(
                FixedUpdate,
                (
                    start_item_use.run_if(on_message::<UseItemRequest>),
                    count_draws.run_if(any_with_component::<Drawing>),
                    release_item_use.run_if(on_message::<ReleaseItemRequest>),
                )
                    .chain()
                    .in_set(TickSet::ClientInput)
                    .after(BlockUseSet),
            )
            .add_systems(
                FixedUpdate,
                (
                    aim_ranged_attacks.run_if(on_message::<RangedAttack>),
                    launch_projectiles.run_if(on_message::<ProjectileLaunch>),
                )
                    .chain()
                    .after(MobBrainSet::Prepare)
                    .before(TickSet::Simulate),
            )
            .add_systems(
                FixedUpdate,
                (
                    fly_projectiles.run_if(any_with_component::<Projectile>),
                    pick_up_arrows.run_if(has_resting_arrows),
                )
                    .chain()
                    .in_set(ProjectileSet),
            )
            .add_systems(
                FixedUpdate,
                report_player_hits
                    .in_set(DamageSet::Report)
                    .ambiguous_with(MobPublish),
            );
    }
}
