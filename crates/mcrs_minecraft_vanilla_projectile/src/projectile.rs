use bevy_ecs::component::Component;
use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_physics::MotionRow;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProjectileKind {
    Arrow,
    Snowball,
}

impl ProjectileKind {
    #[must_use]
    pub fn row(self) -> MotionRow {
        match self {
            Self::Arrow => MotionRow::arrow(),
            Self::Snowball => MotionRow::snowball(),
        }
    }

    #[must_use]
    pub const fn kind(self) -> &'static EntityTypeData {
        match self {
            Self::Arrow => &EntityTypeData::ARROW,
            Self::Snowball => &EntityTypeData::SNOWBALL,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Pickup {
    #[default]
    Disallowed,
    Allowed,
    CreativeOnly,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Projectile {
    pub kind: ProjectileKind,
    pub owner: Option<EntityUuid>,
    pub owner_id: Option<EntityId>,
    pub left_owner: bool,
    pub critical: bool,
    pub base_damage: f64,
    pub speed: f64,
    pub in_ground: bool,
    pub shake: u8,
    pub despawn: u16,
    pub pickup: Pickup,
}

impl Projectile {
    pub const DEFAULT_BASE_DAMAGE: f64 = 2.0;
    pub const SHAKE: u8 = 7;
    pub const DESPAWN_TICKS: u16 = 1200;

    #[must_use]
    pub fn new(kind: ProjectileKind, owner: Option<(EntityUuid, EntityId)>, speed: f64) -> Self {
        Self {
            kind,
            owner: owner.map(|(key, _)| key),
            owner_id: owner.map(|(_, id)| id),
            left_owner: owner.is_none(),
            critical: false,
            base_damage: Self::DEFAULT_BASE_DAMAGE,
            speed,
            in_ground: false,
            shake: 0,
            despawn: 0,
            pickup: Pickup::Disallowed,
        }
    }

    #[must_use]
    pub fn margin(age: u32) -> f64 {
        let age = f64::from(age.min(8));
        ((age - 2.0) / 20.0).clamp(0.0, 0.3)
    }
}
