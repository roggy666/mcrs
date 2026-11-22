use bevy_app::App;
use bevy_app::FixedUpdate;
use bevy_app::Plugin;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::schedule::SystemSet;
use bevy_ecs::schedule::common_conditions::resource_exists_and_changed;
use mcrs_minecraft_damage::Exposure;
use mcrs_minecraft_damage::Fire;
use mcrs_minecraft_engine::PoseSystems;
use mcrs_minecraft_engine_damage::DamageSet;
use mcrs_minecraft_engine_entity_motion::MotionEffects;
use mcrs_minecraft_engine_entity_motion::MotionPass;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_engine_physics::MobPublish;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::tick::TickSet;
use mcrs_minecraft_registries::CurrentRegistries;

use crate::burn::FireDamageTypes;
use crate::burn::bake_fire_damage_types;
use crate::burn::burn;
use crate::exposure::Daylight;
use crate::exposure::sense_exposure;
use crate::publish::BurningShown;
use crate::publish::publish_burning;
use crate::sun::mark_sun_sensitive;
use crate::work::FireWork;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FireSet;

#[derive(Clone, Copy, Debug, Default)]
pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireDamageTypes>();
        app.init_resource::<Daylight>();
        app.init_resource::<FireWork>();
        for result in [
            app.try_register_required_components::<Mob, Fire>(),
            app.try_register_required_components::<Mob, Exposure>(),
            app.try_register_required_components::<Mob, BurningShown>(),
            app.try_register_required_components_with::<Player, Fire>(|| {
                Fire::of(-Fire::PLAYER_GRACE)
            }),
            app.try_register_required_components::<Player, Exposure>(),
        ] {
            if let Err(error) = result {
                tracing::error!("an entity does not require its fire state: {error}");
            }
        }
        app.add_observer(mark_sun_sensitive);
        app.add_systems(
            FixedUpdate,
            bake_fire_damage_types
                .run_if(resource_exists_and_changed::<CurrentRegistries>)
                .in_set(TickSet::DomainInput),
        );
        app.configure_sets(
            FixedUpdate,
            FireSet
                .in_set(MotionPass)
                .after(MotionEffects)
                .after(PoseSystems::Select)
                .before(DamageSet::Collect),
        );
        app.add_systems(FixedUpdate, (sense_exposure, burn).chain().in_set(FireSet));
        app.add_systems(
            FixedUpdate,
            publish_burning
                .in_set(TickSet::Project)
                .before(MobPublish)
                .before(PoseSystems::Publish),
        );
    }
}
