use bevy_ecs::resource::Resource;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use glam::DVec3;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::WriterKey;
use mcrs_minecraft_physics::Body;
use mcrs_minecraft_physics::Rest;

use crate::mob_index::MobIndex;

const KNOCKBACK_LIFT_CAP: f64 = 0.4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntityEffect {
    Impulse(DVec3),
    Knockback { power: f64, direction: DVec3 },
    Place { position: DVec3, velocity: DVec3 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityWrite {
    pub target: EntityUuid,
    pub writer: WriterKey,
    pub sequence: u32,
    pub effect: EntityEffect,
}

impl EntityWrite {
    #[must_use]
    pub const fn impulse(
        target: EntityUuid,
        writer: WriterKey,
        sequence: u32,
        effect: DVec3,
    ) -> Self {
        Self {
            target,
            writer,
            sequence,
            effect: EntityEffect::Impulse(effect),
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct EntityWrites(Vec<EntityWrite>);

impl EntityWrites {
    pub fn push(&mut self, write: EntityWrite) {
        self.0.push(write);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub(crate) fn has_writes(writes: Res<EntityWrites>) -> bool {
    !writes.is_empty()
}

pub(crate) fn apply_entity_writes(
    mut writes: ResMut<EntityWrites>,
    index: Res<MobIndex>,
    mut states: Query<&mut MotionState>,
) {
    let writes = &mut writes.0;
    writes.sort_unstable_by_key(|write| (write.target, write.writer, write.sequence));
    for write in writes.iter() {
        let Some(mut state) =
            (index.get(write.target)).and_then(|entity| states.get_mut(entity).ok())
        else {
            continue;
        };
        let state = &mut state.0;
        apply_effect(&mut state.body, write.effect);
        state.rest = Rest::Awake;
    }
    writes.clear();
}

pub fn apply_effect(body: &mut Body, effect: EntityEffect) {
    match effect {
        EntityEffect::Impulse(effect) => body.velocity += effect,
        EntityEffect::Knockback { power, direction } => {
            let old = body.velocity;
            let push = DVec3::new(direction.x, 0.0, direction.z).normalize_or_zero() * power;
            body.velocity.x = old.x / 2.0 - push.x;
            body.velocity.z = old.z / 2.0 - push.z;
            if body.flags.on_ground {
                body.velocity.y = (old.y / 2.0 + power).min(KNOCKBACK_LIFT_CAP);
            }
        }
        EntityEffect::Place { position, velocity } => {
            body.position = position;
            body.velocity = velocity;
        }
    }
}
