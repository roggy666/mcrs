use bevy_ecs::component::Component;
use glam::DVec3;
use mcrs_minecraft_physics::Steering;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct MobIntent {
    pub steering: Option<Steering>,
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
    pub head_yaw: Option<f32>,
    pub body_yaw: Option<f32>,
    pub velocity: Option<DVec3>,
    pub no_gravity: Option<bool>,
}
