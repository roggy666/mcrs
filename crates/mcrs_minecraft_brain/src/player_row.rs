use glam::DVec3;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;

use crate::HurtBy;
use crate::ItemKind;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerRow {
    pub player: EntityId,
    pub key: EntityUuid,
    pub position: DVec3,
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub flags: PlayerFlags,
    pub visibility: f32,
    pub vehicle: Option<EntityId>,
    pub hands: [ItemKind; 2],
    pub hurt_by: HurtBy,
}

impl PlayerRow {
    pub const DEFAULT_WIDTH: f32 = 0.6;
    pub const DEFAULT_HEIGHT: f32 = 1.8;

    #[must_use]
    pub fn standing(player: EntityId, key: EntityUuid, position: DVec3, eye_height: f32) -> Self {
        Self {
            player,
            key,
            position,
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
            eye_height,
            flags: PlayerFlags::ALIVE,
            visibility: 1.0,
            vehicle: None,
            hands: [ItemKind::EMPTY; 2],
            hurt_by: HurtBy::NONE,
        }
    }

    #[must_use]
    pub fn eye(&self) -> DVec3 {
        DVec3::new(
            self.position.x,
            self.position.y + f64::from(self.eye_height),
            self.position.z,
        )
    }

    #[must_use]
    pub fn is_qualified(&self) -> bool {
        self.flags.contains(PlayerFlags::ALIVE) && !self.flags.contains(PlayerFlags::SPECTATOR)
    }

    #[must_use]
    pub fn is_attackable(&self) -> bool {
        self.is_qualified()
            && !self.flags.contains(PlayerFlags::CREATIVE)
            && !self.flags.contains(PlayerFlags::PEACEFUL)
    }

    #[must_use]
    pub fn holds(&self, test: impl Fn(ItemKind) -> bool) -> bool {
        self.hands.iter().any(|item| test(*item))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PlayerFlags(u8);

impl PlayerFlags {
    pub const NONE: Self = Self(0);
    pub const ALIVE: Self = Self(1);
    pub const SPECTATOR: Self = Self(1 << 1);
    pub const CREATIVE: Self = Self(1 << 2);
    pub const SNEAKING: Self = Self(1 << 3);
    pub const PEACEFUL: Self = Self(1 << 4);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
