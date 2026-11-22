use mcrs_minecraft_physics::BlockFlags;
use mcrs_minecraft_physics::CollisionTable;
use mcrs_minecraft_physics::PathType;
use mcrs_minecraft_physics::ShapeClass;
use mcrs_minecraft_physics::StateCollision;
use mcrs_voxel::BlockStateId;
use rustc_hash::FxHashMap;

use crate::NavId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NavRow {
    pub class: ShapeClass,
    pub flags: BlockFlags,
    pub path: PathType,
}

impl NavRow {
    pub const AIR: Self = Self::of(&StateCollision::AIR);

    pub const UNLOADED: Self = Self {
        class: ShapeClass::FullCube,
        flags: BlockFlags::LEGACY_SOLID,
        path: PathType::Blocked,
    };

    #[must_use]
    pub const fn of(state: &StateCollision) -> Self {
        Self {
            class: state.class,
            flags: state.flags,
            path: state.path,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NavTable {
    rows: Vec<NavRow>,
    representative: Vec<BlockStateId>,
    of_state: Vec<NavId>,
}

impl NavTable {
    #[must_use]
    pub fn new(table: &CollisionTable) -> Self {
        Self::from_rows(
            (0..table.len())
                .filter_map(|index| u16::try_from(index).ok())
                .map(|number| {
                    let id =
                        BlockStateId::checked(number, table.len()).unwrap_or(BlockStateId::AIR);
                    (id, NavRow::of(&table.state(id)))
                }),
        )
    }

    #[must_use]
    pub fn from_rows(states: impl IntoIterator<Item = (BlockStateId, NavRow)>) -> Self {
        let mut rows = vec![NavRow::AIR, NavRow::UNLOADED];
        let mut representative = vec![BlockStateId::AIR, BlockStateId::AIR];
        let mut ids: FxHashMap<NavRow, NavId> = FxHashMap::default();
        ids.insert(NavRow::AIR, NavId::AIR);
        ids.insert(NavRow::UNLOADED, NavId::UNLOADED);
        let mut of_state = Vec::new();
        for (state, row) in states {
            let next = NavId(u16::try_from(rows.len()).unwrap_or(u16::MAX));
            let id = *ids.entry(row).or_insert_with(|| {
                rows.push(row);
                representative.push(state);
                next
            });
            if of_state.len() <= state.index() {
                of_state.resize(state.index() + 1, NavId::AIR);
            }
            if let Some(slot) = of_state.get_mut(state.index()) {
                *slot = id;
            }
        }
        Self {
            rows,
            representative,
            of_state,
        }
    }

    #[must_use]
    pub fn of_state(&self, state: BlockStateId) -> NavId {
        self.of_state
            .get(state.index())
            .copied()
            .unwrap_or(NavId::AIR)
    }

    #[must_use]
    pub fn row(&self, id: NavId) -> NavRow {
        self.rows.get(id.index()).copied().unwrap_or(NavRow::AIR)
    }

    #[must_use]
    pub fn representative(&self, id: NavId) -> BlockStateId {
        self.representative
            .get(id.index())
            .copied()
            .unwrap_or(BlockStateId::AIR)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    pub fn states(&self) -> usize {
        self.of_state.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(number: u16) -> BlockStateId {
        BlockStateId::checked(number, usize::MAX).unwrap()
    }

    fn solid(path: PathType) -> NavRow {
        NavRow {
            class: ShapeClass::FullCube,
            flags: BlockFlags::LEGACY_SOLID | BlockFlags::SOLID_RENDER,
            path,
        }
    }

    #[test]
    fn equal_rows_share_one_id_and_the_first_state_represents_them() {
        let table = NavTable::from_rows([
            (state(0), NavRow::AIR),
            (state(1), solid(PathType::Blocked)),
            (state(2), solid(PathType::Blocked)),
            (state(3), solid(PathType::Lava)),
            (state(4), NavRow::AIR),
        ]);
        assert_eq!(table.len(), 4, "air, unloaded, two solid rows");
        assert_eq!(table.of_state(state(0)), NavId::AIR);
        assert_eq!(table.of_state(state(4)), NavId::AIR);
        assert_eq!(table.of_state(state(1)), table.of_state(state(2)));
        assert_ne!(table.of_state(state(1)), table.of_state(state(3)));
        let shared = table.of_state(state(2));
        assert_eq!(table.representative(shared), state(1));
        assert_eq!(table.row(shared), solid(PathType::Blocked));
        assert_eq!(table.row(NavId::UNLOADED), NavRow::UNLOADED);
    }

    #[test]
    fn an_unknown_state_or_id_reads_as_air() {
        let table = NavTable::from_rows([(state(0), NavRow::AIR)]);
        assert_eq!(table.of_state(state(77)), NavId::AIR);
        assert_eq!(table.row(NavId(99)), NavRow::AIR);
        assert_eq!(table.representative(NavId(99)), BlockStateId::AIR);
    }
}
