#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemKind(pub u16);

impl ItemKind {
    pub const EMPTY: Self = Self(0);

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}
