use glam::DVec3;
use glam::Vec3;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;

use crate::HurtBy;
use crate::ItemKind;
use crate::PlayerFlags;

/// One player of a dimension as the tick sees it: the position at the end of the previous tick,
/// the body box, the flags, the hands and the motion of the last movement packet.
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
    /// The movement of the last movement packet of the client, in blocks per tick. Zero when the
    /// client sent no movement packet in the last tick.
    pub motion: Vec3,
}

const _: () = assert!(
    size_of::<PlayerRow>() <= 128,
    "a player row stays within 128 bytes"
);

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
            motion: Vec3::ZERO,
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

    /// The lower corner of the body box.
    #[must_use]
    pub fn min(&self) -> DVec3 {
        let half = f64::from(self.width) / 2.0;
        DVec3::new(
            self.position.x - half,
            self.position.y,
            self.position.z - half,
        )
    }

    /// The upper corner of the body box.
    #[must_use]
    pub fn max(&self) -> DVec3 {
        let half = f64::from(self.width) / 2.0;
        DVec3::new(
            self.position.x + half,
            self.position.y + f64::from(self.height),
            self.position.z + half,
        )
    }

    /// True when the body box meets the box `(min, max)`, borders included.
    #[must_use]
    pub fn meets(&self, min: DVec3, max: DVec3) -> bool {
        let (own_min, own_max) = (self.min(), self.max());
        own_min.x <= max.x
            && own_max.x >= min.x
            && own_min.y <= max.y
            && own_max.y >= min.y
            && own_min.z <= max.z
            && own_max.z >= min.z
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
