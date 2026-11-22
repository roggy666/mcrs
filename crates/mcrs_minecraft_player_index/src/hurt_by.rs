use mcrs_minecraft_entity_key::EntityId;

use crate::EntityRef;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HurtBy {
    pub attacker: EntityRef,
    pub tick: u32,
}

impl HurtBy {
    pub const NONE: Self = Self {
        attacker: EntityRef::NONE,
        tick: 0,
    };

    #[must_use]
    pub const fn by(attacker: EntityId, tick: u32) -> Self {
        Self {
            attacker: EntityRef::of(attacker),
            tick,
        }
    }

    #[must_use]
    pub const fn get(&self) -> Option<(EntityId, u32)> {
        match self.attacker.get() {
            Some(attacker) => Some((attacker, self.tick)),
            None => None,
        }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.attacker.is_none()
    }
}
