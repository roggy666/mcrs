use mcrs_minecraft_volume::SectionBlocks;
use mcrs_voxel::BlockStateId;
use mcrs_voxel::LocalPos;
use mcrs_voxel::SectionPos;

use crate::NavId;
use crate::NavTable;

const PALETTE: usize = 16;
const BITS: u32 = 4;
const WORDS: usize = SectionPos::VOLUME * BITS as usize / 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavSection {
    Uniform(NavId),
    Packed(Box<Packed>),
    Wide(Box<[NavId; SectionPos::VOLUME]>),
}

const _: () = assert!(
    size_of::<NavSection>() == 16,
    "a uniform section is one id beside the tag; the arrays live behind a box"
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packed {
    palette: [NavId; PALETTE],
    len: u8,
    bits: [u64; WORDS],
}

impl Packed {
    fn of(first: NavId) -> Self {
        let mut palette = [NavId::AIR; PALETTE];
        palette[0] = first;
        Self {
            palette,
            len: 1,
            bits: [0; WORDS],
        }
    }

    fn get(&self, index: usize) -> NavId {
        let word = self.bits.get(index / 16).copied().unwrap_or(0);
        let shift = (index % 16) as u32 * BITS;
        let entry = ((word >> shift) & 0xF) as usize;
        self.palette.get(entry).copied().unwrap_or(NavId::AIR)
    }

    fn entry_of(&self, nav: NavId) -> Option<usize> {
        self.palette
            .get(..usize::from(self.len))?
            .iter()
            .position(|&held| held == nav)
    }

    fn push(&mut self, nav: NavId) -> Option<usize> {
        let at = usize::from(self.len);
        let slot = self.palette.get_mut(at)?;
        *slot = nav;
        self.len += 1;
        Some(at)
    }

    fn write(&mut self, index: usize, entry: usize) {
        let Some(word) = self.bits.get_mut(index / 16) else {
            return;
        };
        let shift = (index % 16) as u32 * BITS;
        *word = (*word & !(0xF << shift)) | ((entry as u64 & 0xF) << shift);
    }

    fn widen(&self) -> Box<[NavId; SectionPos::VOLUME]> {
        let mut wide = Box::new([NavId::AIR; SectionPos::VOLUME]);
        for (index, cell) in wide.iter_mut().enumerate() {
            *cell = self.get(index);
        }
        wide
    }
}

impl NavSection {
    #[must_use]
    pub fn of<S: SectionBlocks + ?Sized>(section: &S, table: &NavTable) -> Self {
        if let Some(state) = section.uniform() {
            return Self::Uniform(table.of_state(state));
        }
        let mut states = Box::new([BlockStateId::AIR; SectionPos::VOLUME]);
        section.copy_to(&mut states);
        let mut cells = Box::new([NavId::AIR; SectionPos::VOLUME]);
        for (cell, state) in cells.iter_mut().zip(states.iter()) {
            *cell = table.of_state(*state);
        }
        Self::from_cells(cells)
    }

    #[must_use]
    pub fn from_cells(cells: Box<[NavId; SectionPos::VOLUME]>) -> Self {
        let Some(&first) = cells.first() else {
            return Self::Uniform(NavId::AIR);
        };
        let mut packed = Packed::of(first);
        for (index, &nav) in cells.iter().enumerate() {
            let entry = match packed.entry_of(nav) {
                Some(entry) => entry,
                None => match packed.push(nav) {
                    Some(entry) => entry,
                    None => return Self::Wide(cells),
                },
            };
            packed.write(index, entry);
        }
        if packed.len == 1 {
            return Self::Uniform(first);
        }
        Self::Packed(Box::new(packed))
    }

    #[must_use]
    pub fn get(&self, at: LocalPos) -> NavId {
        match self {
            Self::Uniform(nav) => *nav,
            Self::Packed(packed) => packed.get(at.index()),
            Self::Wide(cells) => cells.get(at.index()).copied().unwrap_or(NavId::AIR),
        }
    }

    pub fn set(&mut self, at: LocalPos, nav: NavId) {
        let index = at.index();
        match self {
            Self::Uniform(held) => {
                if *held == nav {
                    return;
                }
                let mut packed = Packed::of(*held);
                if let Some(entry) = packed.push(nav) {
                    packed.write(index, entry);
                }
                *self = Self::Packed(Box::new(packed));
            }
            Self::Packed(packed) => {
                if let Some(entry) = packed.entry_of(nav) {
                    packed.write(index, entry);
                    return;
                }
                if let Some(entry) = packed.push(nav) {
                    packed.write(index, entry);
                    return;
                }
                let mut wide = packed.widen();
                if let Some(cell) = wide.get_mut(index) {
                    *cell = nav;
                }
                *self = Self::Wide(wide);
            }
            Self::Wide(cells) => {
                if let Some(cell) = cells.get_mut(index) {
                    *cell = nav;
                }
            }
        }
    }

    #[must_use]
    pub const fn is_uniform(&self) -> bool {
        matches!(self, Self::Uniform(_))
    }

    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        match self {
            Self::Uniform(_) => 0,
            Self::Packed(_) => size_of::<Packed>(),
            Self::Wide(_) => size_of::<[NavId; SectionPos::VOLUME]>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Cells(Box<[BlockStateId; SectionPos::VOLUME]>);

    impl SectionBlocks for Cells {
        fn get(&self, at: LocalPos) -> BlockStateId {
            self.0[at.index()]
        }
    }

    fn state(number: u16) -> BlockStateId {
        BlockStateId::checked(number, usize::MAX).unwrap()
    }

    fn table() -> NavTable {
        use crate::NavRow;
        use mcrs_minecraft_physics::BlockFlags;
        use mcrs_minecraft_physics::PathType;
        use mcrs_minecraft_physics::ShapeClass;
        NavTable::from_rows((0..40u16).map(|number| {
            let row = if number == 0 {
                NavRow::AIR
            } else {
                NavRow {
                    class: ShapeClass::FullCube,
                    flags: BlockFlags::from_bits_retain(u32::from(number)),
                    path: PathType::Blocked,
                }
            };
            (state(number), row)
        }))
    }

    fn pattern(values: u16) -> Cells {
        let mut cells = Box::new([BlockStateId::AIR; SectionPos::VOLUME]);
        for (index, cell) in cells.iter_mut().enumerate() {
            *cell = state((index as u16 * 7 + index as u16 / 13) % values);
        }
        Cells(cells)
    }

    #[test]
    fn a_uniform_section_keeps_no_array() {
        let table = table();
        let section = NavSection::of(&Cells(Box::new([state(3); SectionPos::VOLUME])), &table);
        assert!(section.is_uniform());
        assert_eq!(section.heap_bytes(), 0);
        assert_eq!(
            section.get(LocalPos::new(5, 6, 7)),
            table.of_state(state(3))
        );
    }

    #[test]
    fn up_to_sixteen_values_pack_into_four_bits_and_more_widen() {
        let table = table();
        let cells = pattern(16);
        let packed = NavSection::of(&cells, &table);
        assert!(matches!(packed, NavSection::Packed(_)));
        assert!(packed.heap_bytes() <= 2100, "{}", packed.heap_bytes());
        for at in LocalPos::all() {
            assert_eq!(packed.get(at), table.of_state(cells.get(at)));
        }
        let cells = pattern(17);
        let wide = NavSection::of(&cells, &table);
        assert!(matches!(wide, NavSection::Wide(_)));
        for at in LocalPos::all() {
            assert_eq!(wide.get(at), table.of_state(cells.get(at)));
        }
    }

    #[test]
    fn a_write_promotes_uniform_to_packed_to_wide_and_reads_back() {
        let mut section = NavSection::Uniform(NavId(2));
        section.set(LocalPos::new(1, 1, 1), NavId(2));
        assert!(
            section.is_uniform(),
            "the same value keeps the section uniform"
        );
        section.set(LocalPos::new(1, 1, 1), NavId(3));
        assert!(matches!(section, NavSection::Packed(_)));
        assert_eq!(section.get(LocalPos::new(1, 1, 1)), NavId(3));
        assert_eq!(section.get(LocalPos::new(0, 0, 0)), NavId(2));
        for value in 4..18u16 {
            section.set(LocalPos::new(value as u8, 0, 0), NavId(value));
        }
        assert!(
            matches!(section, NavSection::Packed(_)),
            "sixteen values fit"
        );
        section.set(LocalPos::new(0, 2, 0), NavId(40));
        assert!(matches!(section, NavSection::Wide(_)));
        assert_eq!(section.get(LocalPos::new(0, 2, 0)), NavId(40));
        assert_eq!(section.get(LocalPos::new(9, 0, 0)), NavId(9));
        assert_eq!(section.get(LocalPos::new(1, 1, 1)), NavId(3));
        assert_eq!(section.get(LocalPos::new(15, 15, 15)), NavId(2));
    }
}
