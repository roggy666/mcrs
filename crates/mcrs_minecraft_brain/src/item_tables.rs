use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemKind(pub u16);

impl ItemKind {
    pub const EMPTY: Self = Self(0);

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemTag(pub u16);

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ItemSet {
    words: Vec<u64>,
}

impl ItemSet {
    #[must_use]
    pub fn new(items: impl IntoIterator<Item = ItemKind>) -> Self {
        let mut set = Self::default();
        for item in items {
            set.insert(item);
        }
        set
    }

    pub fn insert(&mut self, item: ItemKind) {
        let word = usize::from(item.0 / 64);
        if self.words.len() <= word {
            self.words.resize(word.saturating_add(1), 0);
        }
        if let Some(slot) = self.words.get_mut(word) {
            *slot |= 1 << (item.0 % 64);
        }
    }

    #[must_use]
    pub fn contains(&self, item: ItemKind) -> bool {
        self.words
            .get(usize::from(item.0 / 64))
            .is_some_and(|word| word & (1 << (item.0 % 64)) != 0)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|word| *word == 0)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ItemTables {
    sets: Vec<ItemSet>,
}

impl ItemTables {
    #[must_use]
    pub fn new(sets: impl IntoIterator<Item = ItemSet>) -> Self {
        Self {
            sets: sets.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn set(&self, tag: ItemTag) -> Option<&ItemSet> {
        self.sets.get(usize::from(tag.0))
    }

    #[must_use]
    pub fn contains(&self, tag: ItemTag, item: ItemKind) -> bool {
        self.set(tag).is_some_and(|set| set.contains(item))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.sets.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sets.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemTest {
    Tag(ItemTag),
    Item(ItemKind),
}

impl ItemTest {
    #[must_use]
    pub fn passes(&self, tables: &ItemTables, item: ItemKind) -> bool {
        match self {
            Self::Tag(tag) => tables.contains(*tag, item),
            Self::Item(kind) => *kind == item,
        }
    }
}
