#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PlayerFlags(u8);

impl PlayerFlags {
    pub const NONE: Self = Self(0);
    pub const ALIVE: Self = Self(1);
    pub const SPECTATOR: Self = Self(1 << 1);
    pub const CREATIVE: Self = Self(1 << 2);
    pub const SNEAKING: Self = Self(1 << 3);
    pub const PEACEFUL: Self = Self(1 << 4);
    pub const ON_GROUND: Self = Self(1 << 5);
    pub const SPRINTING: Self = Self(1 << 6);

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
