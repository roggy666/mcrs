#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NavId(pub u16);

impl NavId {
    pub const AIR: Self = Self(0);
    pub const UNLOADED: Self = Self(1);

    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
