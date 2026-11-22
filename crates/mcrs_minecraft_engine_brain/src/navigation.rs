use bevy_ecs::component::Component;
use mcrs_minecraft_pathfind::Navigation;

#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct MobNavigation(pub Navigation);
