use glam::DVec3;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;

use crate::Dimensions;
use crate::EntityRef;
use crate::HurtBy;
use crate::ItemKind;
use crate::MobFlags;
use crate::MobKind;
use crate::MobView;
use crate::PlayerFlags;
use crate::PlayerRow;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub id: EntityId,
    pub key: EntityUuid,
    pub position: DVec3,
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub visibility: f32,
    pub kind: Option<MobKind>,
    pub flags: CandidateFlags,
    pub owner: EntityRef,
    pub age: i32,
    pub love: u16,
    pub hurt_by: HurtBy,
    pub hands: [ItemKind; 2],
}

impl Candidate {
    #[must_use]
    pub fn mob(view: &MobView, dimensions: Dimensions) -> Self {
        let marks = [
            (view.flags.contains(MobFlags::ALIVE), CandidateFlags::ALIVE),
            (
                view.flags.contains(MobFlags::INVULNERABLE),
                CandidateFlags::INVULNERABLE,
            ),
            (view.flags.contains(MobFlags::BABY), CandidateFlags::BABY),
            (
                view.flags.contains(MobFlags::IN_WATER),
                CandidateFlags::IN_WATER,
            ),
            (view.flags.contains(MobFlags::TAME), CandidateFlags::TAME),
            (
                view.flags.contains(MobFlags::SITTING),
                CandidateFlags::SITTING,
            ),
            (
                view.flags.contains(MobFlags::IN_PANIC),
                CandidateFlags::IN_PANIC,
            ),
            (view.target.is_some(), CandidateFlags::HAS_TARGET),
            (
                view.flags.contains(MobFlags::LEASHED),
                CandidateFlags::LEASHED,
            ),
            (
                view.flags.contains(MobFlags::ON_GROUND),
                CandidateFlags::ON_GROUND,
            ),
            (
                view.flags.contains(MobFlags::HAS_PASSENGER),
                CandidateFlags::HAS_PASSENGER,
            ),
            (
                view.flags.contains(MobFlags::PASSENGER),
                CandidateFlags::PASSENGER,
            ),
        ];
        Self {
            id: view.mob,
            key: view.key,
            position: view.position,
            width: dimensions.width,
            height: dimensions.height,
            eye_height: dimensions.eye_height,
            visibility: 1.0,
            kind: Some(view.kind),
            flags: CandidateFlags::of(marks),
            owner: view.owner,
            age: i32::from(view.age),
            love: view.love,
            hurt_by: view.hurt_by,
            hands: [ItemKind::EMPTY; 2],
        }
    }

    #[must_use]
    pub fn player(row: &PlayerRow) -> Self {
        let marks = [
            (
                row.flags.contains(PlayerFlags::ALIVE),
                CandidateFlags::ALIVE,
            ),
            (
                row.flags.contains(PlayerFlags::SPECTATOR),
                CandidateFlags::SPECTATOR,
            ),
            (
                row.flags.contains(PlayerFlags::CREATIVE),
                CandidateFlags::CREATIVE,
            ),
            (
                row.flags.contains(PlayerFlags::CREATIVE),
                CandidateFlags::INVULNERABLE,
            ),
            (
                row.flags.contains(PlayerFlags::PEACEFUL),
                CandidateFlags::PEACEFUL,
            ),
            (true, CandidateFlags::PLAYER),
        ];
        Self {
            id: row.player,
            key: row.key,
            position: row.position,
            width: row.width,
            height: row.height,
            eye_height: row.eye_height,
            visibility: row.visibility,
            kind: None,
            flags: CandidateFlags::of(marks),
            owner: EntityRef::NONE,
            age: 0,
            love: 0,
            hurt_by: row.hurt_by,
            hands: row.hands,
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
    pub fn is_alive(&self) -> bool {
        self.flags.contains(CandidateFlags::ALIVE)
    }

    #[must_use]
    pub fn is_player(&self) -> bool {
        self.flags.contains(CandidateFlags::PLAYER)
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.age < 0
    }

    #[must_use]
    pub fn in_love(&self) -> bool {
        self.love > 0
    }

    #[must_use]
    pub fn box_min(&self) -> DVec3 {
        let half = f64::from(self.width) * 0.5;
        DVec3::new(
            self.position.x - half,
            self.position.y,
            self.position.z - half,
        )
    }

    #[must_use]
    pub fn box_max(&self) -> DVec3 {
        let half = f64::from(self.width) * 0.5;
        DVec3::new(
            self.position.x + half,
            self.position.y + f64::from(self.height),
            self.position.z + half,
        )
    }

    #[must_use]
    pub fn intersects(&self, min: DVec3, max: DVec3) -> bool {
        let low = self.box_min();
        let high = self.box_max();
        low.x <= max.x
            && min.x <= high.x
            && low.y <= max.y
            && min.y <= high.y
            && low.z <= max.z
            && min.z <= high.z
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CandidateFlags(u16);

impl CandidateFlags {
    pub const NONE: Self = Self(0);
    pub const ALIVE: Self = Self(1);
    pub const SPECTATOR: Self = Self(1 << 1);
    pub const CREATIVE: Self = Self(1 << 2);
    pub const INVULNERABLE: Self = Self(1 << 3);
    pub const PLAYER: Self = Self(1 << 4);
    pub const BABY: Self = Self(1 << 5);
    pub const IN_WATER: Self = Self(1 << 6);
    pub const TAME: Self = Self(1 << 7);
    pub const SITTING: Self = Self(1 << 8);
    pub const IN_PANIC: Self = Self(1 << 9);
    pub const HAS_TARGET: Self = Self(1 << 10);
    pub const LEASHED: Self = Self(1 << 11);
    pub const PEACEFUL: Self = Self(1 << 12);
    pub const ON_GROUND: Self = Self(1 << 13);
    pub const HAS_PASSENGER: Self = Self(1 << 14);
    pub const PASSENGER: Self = Self(1 << 15);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[must_use]
    pub fn of(marks: impl IntoIterator<Item = (bool, Self)>) -> Self {
        marks
            .into_iter()
            .filter(|(set, _)| *set)
            .fold(Self::NONE, |flags, (_, flag)| flags.union(flag))
    }
}
