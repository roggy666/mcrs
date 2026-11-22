use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;

#[derive(Resource, Clone, Debug, Default)]
pub struct FireWork(Vec<Entity>);

impl FireWork {
    pub fn insert(&mut self, entity: Entity) {
        if !self.0.contains(&entity) {
            self.0.push(entity);
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn entities(&self) -> &[Entity] {
        &self.0
    }

    pub fn retain(&mut self, keep: impl FnMut(&Entity) -> bool) {
        self.0.retain(keep);
    }
}
