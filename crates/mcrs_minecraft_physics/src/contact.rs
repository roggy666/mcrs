use crate::BlockFlags;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Contact: u8 {
        const WATER = 1 << 0;
        const LAVA = 1 << 1;
        const FIRE = 1 << 2;
        const SOUL_FIRE = 1 << 3;
    }
}

impl Contact {
    #[must_use]
    pub fn of_block(flags: BlockFlags) -> Self {
        let mut contact = Self::empty();
        if flags.contains(BlockFlags::WATER) {
            contact |= Self::WATER;
        }
        if flags.contains(BlockFlags::LAVA) {
            contact |= Self::LAVA;
        }
        if flags.contains(BlockFlags::FIRE) {
            contact |= Self::FIRE;
        }
        if flags.contains(BlockFlags::SOUL_FIRE) {
            contact |= Self::SOUL_FIRE;
        }
        contact
    }

    #[must_use]
    pub const fn in_water(self) -> bool {
        self.contains(Self::WATER)
    }

    #[must_use]
    pub const fn in_lava(self) -> bool {
        self.contains(Self::LAVA)
    }

    #[must_use]
    pub const fn in_fire(self) -> bool {
        self.intersects(Self::FIRE.union(Self::SOUL_FIRE))
    }
}
