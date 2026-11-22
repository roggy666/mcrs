use core::num::NonZeroU64;

use mcrs_minecraft_entity_key::EntityId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EntityRef(Option<NonZeroU64>);

impl EntityRef {
    pub const NONE: Self = Self(None);

    #[must_use]
    pub const fn of(id: EntityId) -> Self {
        match id.0.checked_add(1) {
            Some(shifted) => Self(NonZeroU64::new(shifted)),
            None => Self::NONE,
        }
    }

    #[must_use]
    pub fn some(id: Option<EntityId>) -> Self {
        match id {
            Some(id) => Self::of(id),
            None => Self::NONE,
        }
    }

    #[must_use]
    pub const fn get(&self) -> Option<EntityId> {
        match self.0 {
            Some(shifted) => Some(EntityId(shifted.get() - 1)),
            None => None,
        }
    }

    #[must_use]
    pub const fn is_some(&self) -> bool {
        self.0.is_some()
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.0.is_none()
    }

    #[must_use]
    pub fn is(&self, id: EntityId) -> bool {
        self.get() == Some(id)
    }
}

impl From<EntityId> for EntityRef {
    fn from(id: EntityId) -> Self {
        Self::of(id)
    }
}

impl From<Option<EntityId>> for EntityRef {
    fn from(id: Option<EntityId>) -> Self {
        Self::some(id)
    }
}
