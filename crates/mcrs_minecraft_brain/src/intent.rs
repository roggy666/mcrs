use glam::DVec3;
use mcrs_minecraft_entity_key::EntityId;

use crate::Proposal;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ticket(pub u32);

impl Ticket {
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Intents {
    pub path: Option<PathIntent>,
    pub look: Option<LookIntent>,
    pub panic: Option<Panic>,
    pub target: Option<TargetIntent>,
    pub melee: Option<MeleeIntent>,
    pub ranged: Option<RangedIntent>,
    pub leap: Option<Leap>,
    pub speed: Option<f32>,
    pub pose: Option<PoseIntent>,
    pub proposal: Option<Proposal>,
}

impl Intents {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathIntent {
    Walk {
        ticket: Ticket,
        target: DVec3,
        speed_modifier: f32,
    },
    Clear {
        ticket: Ticket,
    },
}

impl PathIntent {
    #[must_use]
    pub const fn ticket(&self) -> Ticket {
        match self {
            Self::Walk { ticket, .. } | Self::Clear { ticket } => *ticket,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LookIntent {
    At {
        point: DVec3,
        turn_speed: Option<f32>,
        pitch_limit: Option<f32>,
        hold: u8,
    },
    Around {
        cos: f32,
        sin: f32,
    },
    EndAround,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Panic {
    #[default]
    Calm,
    Panicking,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetIntent {
    Set(EntityId),
    Clear,
}

impl TargetIntent {
    #[must_use]
    pub const fn get(&self) -> Option<EntityId> {
        match self {
            Self::Set(target) => Some(*target),
            Self::Clear => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeleeIntent {
    Start {
        speed_modifier: f32,
        interval: u8,
        pursuit: Pursuit,
    },
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leap {
    pub toward_x: f32,
    pub toward_z: f32,
    pub vertical: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pursuit {
    Seen,
    Unseen,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangedIntent {
    Start {
        speed_modifier: f32,
        min_interval: u16,
        max_interval: u16,
        range: f32,
    },
    Stop,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PoseBits(u8);

impl PoseBits {
    pub const NONE: Self = Self(0);
    pub const AGGRESSIVE: Self = Self(1);
    pub const SITTING: Self = Self(1 << 1);
    pub const INTERESTED: Self = Self(1 << 2);
    pub const SWELLING: Self = Self(1 << 3);
    pub const EATING: Self = Self(1 << 4);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PoseIntent {
    pub set: PoseBits,
    pub clear: PoseBits,
}

impl PoseIntent {
    #[must_use]
    pub const fn set(bits: PoseBits) -> Self {
        Self {
            set: bits,
            clear: PoseBits::NONE,
        }
    }

    #[must_use]
    pub const fn clear(bits: PoseBits) -> Self {
        Self {
            set: PoseBits::NONE,
            clear: bits,
        }
    }

    #[must_use]
    pub const fn merged(self, other: Self) -> Self {
        Self {
            set: self.set.union(other.set),
            clear: self.clear.union(other.clear),
        }
    }
}
