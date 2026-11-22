use mcrs_minecraft_path_type::PathType;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CellBits(u8);

impl CellBits {
    pub const NONE: Self = Self(0);
    pub const AIR: Self = Self(1);
    pub const SHAPE: Self = Self(1 << 1);
    pub const SOLID: Self = Self(1 << 2);
    pub const FLOATABLE: Self = Self(1 << 3);
    pub const CAULDRON: Self = Self(1 << 4);
    pub const DOOR_OR_FENCE: Self = Self(1 << 5);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[must_use]
    pub const fn with(self, flag: Self, set: bool) -> Self {
        if set { self.union(flag) } else { self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathCell {
    pub kind: PathType,
    pub top: f32,
    pub bits: CellBits,
}

impl PathCell {
    pub const AIR: Self = Self {
        kind: PathType::Open,
        top: 0.0,
        bits: CellBits::AIR,
    };

    #[must_use]
    pub const fn is_air(&self) -> bool {
        self.bits.contains(CellBits::AIR)
    }

    #[must_use]
    pub const fn passable_on_land(&self) -> bool {
        !self.bits.contains(CellBits::SHAPE) && !self.bits.contains(CellBits::DOOR_OR_FENCE)
    }

    #[must_use]
    pub const fn is_solid(&self) -> bool {
        self.bits.contains(CellBits::SOLID)
    }

    #[must_use]
    pub const fn floatable(&self) -> bool {
        self.bits.contains(CellBits::FLOATABLE)
    }
}
