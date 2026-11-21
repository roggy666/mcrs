#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MobKind(pub u16);

const WORDS: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KindSet([u64; WORDS]);

#[expect(
    clippy::indexing_slicing,
    reason = "each index is tested against the fixed word count before the read, in const code that has no get"
)]
impl KindSet {
    pub const NONE: Self = Self([0; WORDS]);
    pub const ALL: Self = Self([u64::MAX; WORDS]);
    pub const CAPACITY: u16 = (WORDS * 64) as u16;

    #[must_use]
    pub const fn of(kinds: &[MobKind]) -> Self {
        let mut set = Self::NONE;
        let mut at = 0;
        while at < kinds.len() {
            set = set.with(kinds[at]);
            at += 1;
        }
        set
    }

    #[must_use]
    pub const fn with(self, kind: MobKind) -> Self {
        let mut words = self.0;
        let word = (kind.0 / 64) as usize;
        if word < WORDS {
            words[word] |= 1 << (kind.0 % 64);
        }
        Self(words)
    }

    #[must_use]
    pub const fn contains(&self, kind: MobKind) -> bool {
        let word = (kind.0 / 64) as usize;
        word < WORDS && self.0[word] & (1 << (kind.0 % 64)) != 0
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        let mut words = self.0;
        let mut at = 0;
        while at < WORDS {
            words[at] |= other.0[at];
            at += 1;
        }
        Self(words)
    }

    #[must_use]
    pub const fn intersects(&self, other: &Self) -> bool {
        let mut at = 0;
        while at < WORDS {
            if self.0[at] & other.0[at] != 0 {
                return true;
            }
            at += 1;
        }
        false
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        let mut at = 0;
        while at < WORDS {
            if self.0[at] != 0 {
                return false;
            }
            at += 1;
        }
        true
    }
}
