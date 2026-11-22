use alloc::vec::Vec;

use glam::DVec3;
use hashbrown::HashMap;
use mcrs_minecraft_path_type::PathType;
use mcrs_voxel::BlockPos;

use crate::CellBits;
use crate::MobPath;
use crate::PathCell;
use crate::PathSearchWorld;
use crate::PathWorld;
use crate::Request;
use crate::Searcher;
use crate::search;

pub(crate) struct MapWorld {
    cells: HashMap<(i32, i32, i32), PathCell>,
    floor_y: i32,
}

pub(crate) const STONE: PathCell = PathCell {
    kind: PathType::Blocked,
    top: 1.0,
    bits: CellBits::SHAPE.union(CellBits::SOLID),
};

pub(crate) const WATER: PathCell = PathCell {
    kind: PathType::Water,
    top: 0.0,
    bits: CellBits::FLOATABLE,
};

pub(crate) const FENCE: PathCell = PathCell {
    kind: PathType::Fence,
    top: 1.5,
    bits: CellBits::SHAPE.union(CellBits::DOOR_OR_FENCE),
};

pub(crate) const LAVA: PathCell = PathCell {
    kind: PathType::Lava,
    top: 0.0,
    bits: CellBits::NONE,
};

impl MapWorld {
    pub(crate) fn flat(floor_y: i32) -> Self {
        Self {
            cells: HashMap::new(),
            floor_y,
        }
    }

    pub(crate) fn set(&mut self, at: BlockPos, cell: PathCell) -> &mut Self {
        self.cells.insert((at.x, at.y, at.z), cell);
        self
    }

    pub(crate) fn clear(&mut self, at: BlockPos) -> &mut Self {
        self.cells.insert((at.x, at.y, at.z), PathCell::AIR);
        self
    }

    pub(crate) fn wall(&mut self, x: i32, z0: i32, z1: i32, y0: i32, y1: i32) -> &mut Self {
        for z in z0..=z1 {
            for y in y0..=y1 {
                self.set(BlockPos::new(x, y, z), STONE);
            }
        }
        self
    }

    fn solid_at(&self, at: BlockPos) -> bool {
        self.path_cell(at).bits.contains(CellBits::SHAPE)
    }
}

impl PathWorld for MapWorld {
    fn path_cell(&self, at: BlockPos) -> PathCell {
        if let Some(cell) = self.cells.get(&(at.x, at.y, at.z)) {
            return *cell;
        }
        if at.y <= self.floor_y {
            STONE
        } else {
            PathCell::AIR
        }
    }

    fn min_y(&self) -> i32 {
        -64
    }

    fn max_y(&self) -> i32 {
        319
    }

    fn column_loaded(&self, _: BlockPos) -> bool {
        true
    }

    fn collides(&self, min: DVec3, max: DVec3) -> bool {
        let low = BlockPos::from(min);
        let high = BlockPos::from(max - DVec3::splat(1.0e-7));
        for x in low.x..=high.x {
            for y in low.y..=high.y {
                for z in low.z..=high.z {
                    let at = BlockPos::new(x, y, z);
                    if !self.solid_at(at) {
                        continue;
                    }
                    let cell = self.path_cell(at);
                    let top = f64::from(y) + f64::from(cell.top);
                    if min.y < top && max.y > f64::from(y) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub(crate) const ZOMBIE_FOLLOW_RANGE: f32 = 35.0;

pub(crate) fn near(path: &[BlockPos], target: BlockPos) -> bool {
    path.last().is_some_and(|end| {
        (end.x - target.x).abs() + (end.y - target.y).abs() + (end.z - target.z).abs() <= 1
    })
}

impl PathSearchWorld for MapWorld {
    type Search<'a> = &'a MapWorld;

    fn open(&self, _: &MobPath) -> Self::Search<'_> {
        self
    }
}

pub(crate) fn zombie_at(x: f64, y: f64, z: f64) -> MobPath {
    MobPath::standing(DVec3::new(x, y, z), 0.6, 1.95, 0.6)
}

pub(crate) fn walk(world: &MapWorld, mob: &MobPath, to: BlockPos) -> Option<Vec<BlockPos>> {
    let request = Request::new(
        crate::RequestKind::Blocks,
        alloc::vec![to],
        Request::DEFAULT_REACH,
        1.0,
        ZOMBIE_FOLLOW_RANGE,
    );
    let mut searcher = Searcher::default();
    let mut world = world;
    let outcome = search(&mut world, mob, &request, &mut searcher);
    outcome
        .path
        .filter(|path| path.reached)
        .map(|path| path.nodes.iter().map(|node| node.pos()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_on_a_plain_walks_straight_to_the_target() {
        let world = MapWorld::flat(63);
        let mob = zombie_at(0.5, 64.0, 0.5);
        let path = walk(&world, &mob, BlockPos::new(5, 64, 0)).expect("a reached path");
        assert_eq!(path.first().copied(), Some(BlockPos::new(0, 64, 0)));
        assert!(near(&path, BlockPos::new(5, 64, 0)), "{path:?}");
        assert!(path.iter().all(|node| node.y == 64 && node.z == 0));
        assert_eq!(path.len(), 5);
    }

    #[test]
    fn a_search_in_slices_of_three_pops_finds_the_path_of_one_search() {
        let mut world = MapWorld::flat(63);
        world.wall(4, -4, 4, 64, 65);
        let mob = zombie_at(0.5, 64.0, 0.5);
        let target = BlockPos::new(9, 64, 0);
        let whole = walk(&world, &mob, target).expect("a reached path");
        let request = Request::new(
            crate::RequestKind::Blocks,
            alloc::vec![target],
            Request::DEFAULT_REACH,
            1.0,
            ZOMBIE_FOLLOW_RANGE,
        );
        let mut searcher = Searcher::default();
        let mut view = &world;
        assert!(crate::search_begin(
            &mut view,
            &mob,
            &request,
            &mut searcher
        ));
        let mut slices = 0;
        let outcome = loop {
            slices += 1;
            match crate::search_advance(&mut view, &mob, &request, &mut searcher, 3) {
                crate::SearchStep::Suspended { spent } => assert_eq!(spent, 3),
                crate::SearchStep::Done { outcome, spent } => {
                    assert!(spent <= 3);
                    break outcome;
                }
            }
        };
        assert!(slices > 1, "the search took more than one slice: {slices}");
        let sliced: Vec<BlockPos> = outcome
            .path
            .filter(|path| path.reached)
            .expect("a reached path")
            .nodes
            .iter()
            .map(|node| node.pos())
            .collect();
        assert_eq!(sliced, whole);
    }

    #[test]
    fn a_wall_with_a_gap_bends_the_path_through_the_gap() {
        let mut world = MapWorld::flat(63);
        world.wall(3, -6, 6, 64, 66);
        world.clear(BlockPos::new(3, 64, 4));
        world.clear(BlockPos::new(3, 65, 4));
        let mob = zombie_at(0.5, 64.0, 0.5);
        let path = walk(&world, &mob, BlockPos::new(6, 64, 0)).expect("a reached path");
        assert!(path.contains(&BlockPos::new(3, 64, 4)), "{path:?}");
        assert!(path.iter().all(|node| node.y == 64));
    }

    #[test]
    fn a_step_of_one_block_is_climbed_and_a_ledge_of_two_is_not() {
        let mut world = MapWorld::flat(63);
        for z in -3..=3 {
            world.set(BlockPos::new(3, 64, z), STONE);
            world.set(BlockPos::new(4, 64, z), STONE);
        }
        let mob = zombie_at(0.5, 64.0, 0.5);
        let path = walk(&world, &mob, BlockPos::new(4, 65, 0)).expect("a reached path");
        assert!(near(&path, BlockPos::new(4, 65, 0)), "{path:?}");
        assert_eq!(path.last().map(|end| end.y), Some(65));
        for z in -3..=3 {
            world.set(BlockPos::new(3, 65, z), STONE);
            world.set(BlockPos::new(4, 65, z), STONE);
        }
        assert!(walk(&world, &mob, BlockPos::new(4, 66, 0)).is_none());
    }

    #[test]
    fn a_drop_of_three_is_taken_and_a_drop_of_four_is_not() {
        let mut world = MapWorld::flat(63);
        for x in 2..=8 {
            for z in -3..=3 {
                for y in 61..=63 {
                    world.clear(BlockPos::new(x, y, z));
                }
            }
        }
        let mob = zombie_at(0.5, 64.0, 0.5);
        let path = walk(&world, &mob, BlockPos::new(5, 61, 0)).expect("a drop of three");
        assert!(near(&path, BlockPos::new(5, 61, 0)), "{path:?}");
        assert_eq!(path.last().map(|end| end.y), Some(61));
        for x in 2..=8 {
            for z in -3..=3 {
                world.clear(BlockPos::new(x, 60, z));
            }
        }
        assert!(walk(&world, &mob, BlockPos::new(5, 60, 0)).is_none());
    }

    #[test]
    fn a_fence_is_walked_around_and_lava_is_avoided() {
        let mut world = MapWorld::flat(63);
        for z in -8..=8 {
            world.set(BlockPos::new(3, 64, z), FENCE);
        }
        let mob = zombie_at(0.5, 64.0, 0.5);
        let path = walk(&world, &mob, BlockPos::new(6, 64, 0)).expect("a path around the fence");
        assert!(
            path.iter().all(|node| !(node.x == 3 && node.z.abs() <= 8)),
            "{path:?}"
        );
        let mut world = MapWorld::flat(63);
        for z in -1..=1 {
            world.set(BlockPos::new(3, 63, z), LAVA);
        }
        let path = walk(&world, &mob, BlockPos::new(6, 64, 0)).expect("a path around the lava");
        assert!(
            path.iter().all(|node| !(node.x == 3 && node.z.abs() <= 1)),
            "{path:?}"
        );
    }

    #[test]
    fn water_is_waded_with_its_malus() {
        let mut world = MapWorld::flat(63);
        for z in -8..=8 {
            world.set(BlockPos::new(3, 64, z), WATER);
            world.clear(BlockPos::new(3, 63, z));
            world.set(BlockPos::new(3, 63, z), WATER);
        }
        let mob = zombie_at(0.5, 64.0, 0.5);
        let path = walk(&world, &mob, BlockPos::new(6, 64, 0)).expect("a path through water");
        assert!(path.iter().any(|node| node.x == 3), "{path:?}");
    }
}
