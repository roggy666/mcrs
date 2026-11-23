#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Generation {
    Original,
    Copy,
    CopyOfCopy,
    Tattered,
}

impl Generation {
    pub(crate) const ALL: [Self; 4] =
        [Self::Original, Self::Copy, Self::CopyOfCopy, Self::Tattered];

    #[must_use]
    pub const fn id(self) -> i32 {
        match self {
            Self::Original => 0,
            Self::Copy => 1,
            Self::CopyOfCopy => 2,
            Self::Tattered => 3,
        }
    }

    #[must_use]
    pub fn from_id(id: i32) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|generation| generation.id() == id)
    }
}
