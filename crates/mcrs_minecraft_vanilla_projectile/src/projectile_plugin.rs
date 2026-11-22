use bevy_app::App;
use bevy_app::FixedUpdate;
use bevy_app::Plugin;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::schedule::SystemSet;
use bevy_ecs::schedule::common_conditions::any_with_component;
use bevy_ecs::schedule::common_conditions::on_message;
use bevy_ecs::schedule::common_conditions::resource_exists_and_changed;
use mcrs_minecraft_engine_brain::MobBrainSet;
use mcrs_minecraft_engine_brain::RangedAttack;
use mcrs_minecraft_engine_damage::DamageSet;
use mcrs_minecraft_engine_entity_motion::MotionPass;
use mcrs_minecraft_engine_world::tick::TickSet;
use mcrs_minecraft_registries::CurrentRegistries;

use crate::Projectile;
use crate::ProjectileLaunch;
use crate::flight::ProjectileDamageTypes;
use crate::flight::bake_damage_types;
use crate::flight::fly_projectiles;
use crate::launch::aim_ranged_attacks;
use crate::launch::launch_projectiles;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProjectilePlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectileSet;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectileDamageTypes>()
            .add_message::<ProjectileLaunch>()
            .add_message::<RangedAttack>()
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
                    aim_ranged_attacks.run_if(on_message::<RangedAttack>),
                    launch_projectiles.run_if(on_message::<ProjectileLaunch>),
                )
                    .chain()
                    .after(MobBrainSet::Prepare)
                    .before(TickSet::Simulate),
            )
            .add_systems(
                FixedUpdate,
                fly_projectiles
                    .run_if(any_with_component::<Projectile>)
                    .in_set(ProjectileSet),
            );
    }
}
