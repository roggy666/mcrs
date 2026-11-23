use mcrs_minecraft_path_type::PathType;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GroundCell {
    pub flags: CellFlags,
    pub malus: f32,
    pub walk_value: f32,
    pub path: PathType,
}

impl GroundCell {
    #[must_use]
    pub fn is_walkable(&self) -> bool {
        self.flags.contains(CellFlags::STABLE_FLOOR)
            && !self.flags.contains(CellFlags::LEGACY_SOLID)
            && self.malus == 0.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CellFlags(u16);

impl CellFlags {
    pub const NONE: Self = Self(0);
    pub const STABLE_FLOOR: Self = Self(1);
    pub const LEGACY_SOLID: Self = Self(1 << 1);
    pub const WATER: Self = Self(1 << 2);
    pub const NO_COLLISION: Self = Self(1 << 3);
    pub const GRASS_BELOW: Self = Self(1 << 4);
    pub const EDIBLE: Self = Self(1 << 5);
    pub const AIR: Self = Self(1 << 6);
    pub const AIR_BELOW: Self = Self(1 << 7);
    pub const LEAVES_BELOW: Self = Self(1 << 8);
    pub const LOG_BELOW: Self = Self(1 << 9);
    pub const STANDABLE: Self = Self(1 << 10);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BuildHeight {
    pub min: i32,
    pub max: i32,
}

impl BuildHeight {
    #[must_use]
    pub const fn contains(&self, y: i32) -> bool {
        self.min <= y && y <= self.max
    }
}
