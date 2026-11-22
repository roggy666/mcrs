use crate::ControlFlags;
use crate::EntityRef;
use crate::MobKind;
use crate::Ticket;
use glam::DVec3;
use glam::Vec3;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MobView {
    pub mob: EntityId,
    pub key: EntityUuid,
    pub position: DVec3,
    pub velocity: Vec3,
    pub kind: MobKind,
    pub flags: MobFlags,
    pub health: f32,
    pub follow_range: f32,
    pub age: i32,
    pub love: u16,
    pub idle_time: u16,
    pub damage: Option<DamageRecord>,
    pub hurt_by: HurtBy,
    pub target: EntityRef,
    pub owner: EntityRef,
    pub path_ticket: Ticket,
}

const _: () = assert!(
    size_of::<MobView>() <= 128,
    "a view is copied for each mob in each tick and kept for max_lag versions: two cache lines"
);

impl MobView {
    #[must_use]
    pub fn eye(&self, eye_height: f32) -> DVec3 {
        DVec3::new(
            self.position.x,
            self.position.y + f64::from(eye_height),
            self.position.z,
        )
    }

    #[must_use]
    pub const fn path(&self) -> Applied {
        Applied {
            ticket: self.path_ticket,
            progress: if self.flags.contains(MobFlags::PATH_RUNNING) {
                Progress::Running
            } else {
                Progress::Ended
            },
        }
    }

    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.flags.contains(MobFlags::ALIVE)
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.age < 0
    }

    #[must_use]
    pub fn in_love(&self) -> bool {
        self.love > 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MobFlags(u32);

impl MobFlags {
    pub const NONE: Self = Self(0);
    pub const ALIVE: Self = Self(1);
    pub const ON_GROUND: Self = Self(1 << 1);
    pub const IN_WATER: Self = Self(1 << 2);
    pub const ON_FIRE: Self = Self(1 << 3);
    pub const CONTROLLED: Self = Self(1 << 4);
    pub const BABY: Self = Self(1 << 5);
    pub const IN_PANIC: Self = Self(1 << 6);
    pub const TAME: Self = Self(1 << 7);
    pub const SITTING: Self = Self(1 << 8);
    pub const LEASHED: Self = Self(1 << 9);
    pub const PASSENGER: Self = Self(1 << 10);
    pub const ANGRY: Self = Self(1 << 11);
    pub const IN_LAVA: Self = Self(1 << 12);
    pub const INVULNERABLE: Self = Self(1 << 13);
    pub const HAS_PASSENGER: Self = Self(1 << 14);
    pub const PATH_RUNNING: Self = Self(1 << 15);
    pub const SWELLING: Self = Self(1 << 16);
    pub const AFLOAT: Self = Self(1 << 17);
    pub const IN_SUN: Self = Self(1 << 18);
    pub const MOVE_DISABLED: Self = Self(1 << 19);
    pub const LOOK_DISABLED: Self = Self(1 << 20);
    pub const JUMP_DISABLED: Self = Self(1 << 21);
    pub const EATING: Self = Self(1 << 22);
    pub const SPIT: Self = Self(1 << 23);
    pub const IN_CARAVAN: Self = Self(1 << 24);
    pub const HAS_TAIL: Self = Self(1 << 25);

    #[must_use]
    pub const fn disabled(self) -> ControlFlags {
        let mut disabled = ControlFlags::NONE;
        if self.contains(Self::MOVE_DISABLED) {
            disabled = disabled.union(ControlFlags::MOVE);
        }
        if self.contains(Self::LOOK_DISABLED) {
            disabled = disabled.union(ControlFlags::LOOK);
        }
        if self.contains(Self::JUMP_DISABLED) {
            disabled = disabled.union(ControlFlags::JUMP);
        }
        disabled
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[must_use]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[must_use]
    pub fn of(marks: impl IntoIterator<Item = (bool, Self)>) -> Self {
        marks
            .into_iter()
            .filter(|(set, _)| *set)
            .fold(Self::NONE, |flags, (_, flag)| flags.union(flag))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DamageRecord {
    pub tick: u32,
    pub cause: DamageCause,
}

impl DamageRecord {
    #[must_use]
    pub const fn age(&self, now: u64) -> u64 {
        (now as u32).wrapping_sub(self.tick) as u64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DamageCause {
    Panic,
    Other,
}

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Applied {
    pub ticket: Ticket,
    pub progress: Progress,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Progress {
    #[default]
    Ended,
    Running,
}
