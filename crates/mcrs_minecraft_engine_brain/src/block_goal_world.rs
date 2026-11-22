use glam::DVec3;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_goal_world::CellFlags;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_minecraft_goal_world::GroundCell;
use mcrs_minecraft_pathfind::CellBits;
use mcrs_minecraft_pathfind::MobPath;
use mcrs_minecraft_pathfind::PathCell;
use mcrs_minecraft_pathfind::PathSearchWorld;
use mcrs_minecraft_pathfind::PathWorld;
use mcrs_minecraft_physics::BlockFlags;
use mcrs_minecraft_physics::CollisionTable;
use mcrs_minecraft_physics::PathType;
use mcrs_minecraft_physics::ShapeClass;
use mcrs_minecraft_physics::StateCollision;
use mcrs_minecraft_physics::Tables;
use mcrs_minecraft_physics::sight_clear;
use mcrs_minecraft_volume::BlockVolume;
use mcrs_minecraft_volume::Unavailable;
use mcrs_voxel::BlockPos;
use mcrs_voxel::SectionPos;

use crate::SearchView;
use crate::sight_blocks::SightBlocks;

const GRASS_WALK_VALUE: f32 = 10.0;
const OTHER_WALK_VALUE: f32 = 0.5;
const SIGHT_RANGE: f64 = 128.0;

const UNLOADED: StateCollision = StateCollision {
    class: ShapeClass::FullCube,
    flags: BlockFlags::LEGACY_SOLID,
    path: PathType::Blocked,
    ..StateCollision::AIR
};

#[derive(Debug)]
pub struct BlockGoalWorld<'a, V> {
    blocks: &'a V,
    tables: &'a Tables,
    table: &'a CollisionTable,
    height: BuildHeight,
}

impl<'a, V: BlockVolume> BlockGoalWorld<'a, V> {
    #[must_use]
    pub fn new(blocks: &'a V, tables: &'a Tables) -> Self {
        let bounds = blocks.bounds();
        Self {
            blocks,
            tables,
            table: &tables.collision,
            height: BuildHeight {
                min: bounds.min.y,
                max: bounds.max.y,
            },
        }
    }

    #[must_use]
    pub(crate) fn blocks(&self) -> &'a V {
        self.blocks
    }

    #[must_use]
    pub(crate) fn tables(&self) -> &'a Tables {
        self.tables
    }

    #[must_use]
    pub(crate) fn unloaded_cell(&self) -> PathCell {
        path_cell_of(self.table, UNLOADED)
    }

    #[must_use]
    pub fn is_cauldron(&self, at: BlockPos) -> bool {
        self.state(at).flags.contains(BlockFlags::CAULDRON)
    }

    fn state(&self, at: BlockPos) -> StateCollision {
        match self
            .blocks
            .block(mcrs_voxel::BlockPos::new(at.x, at.y, at.z))
        {
            Ok(state) => self.table.state(state),
            Err(Unavailable::Outside) => StateCollision::AIR,
            Err(Unavailable::Unloaded | Unavailable::Foreign) => UNLOADED,
        }
    }

    #[must_use]
    pub fn feet_top(&self, at: BlockPos) -> Option<f64> {
        let state = self.state(at);
        if state.flags.intersects(BlockFlags::DOOR | BlockFlags::FENCE) {
            return None;
        }
        let shape = self.table.shape_without_context(state.class)?;
        Some(f64::from(at.y) + shape.top())
    }

    fn block_path_type(
        &self,
        at: BlockPos,
        here: StateCollision,
        below: StateCollision,
    ) -> PathType {
        if here.path != PathType::Open || at.y <= self.height.min {
            return here.path;
        }
        if let Some(path) = PathType::over(below.path) {
            return path;
        }
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dz == 0 {
                        continue;
                    }
                    let neighbour = self.state(at.offset(dx, dy, dz)).path;
                    if let Some(path) = neighbour.as_neighbour() {
                        return path;
                    }
                }
            }
        }
        PathType::Walkable
    }
}

impl<V: BlockVolume> GoalWorld for BlockGoalWorld<'_, V> {
    fn build_height(&self) -> BuildHeight {
        self.height
    }

    fn cell(&self, at: BlockPos) -> GroundCell {
        let here = self.state(at);
        let below = self.state(at.below());
        let marks = [
            (
                below.flags.contains(BlockFlags::SOLID_RENDER),
                CellFlags::STABLE_FLOOR,
            ),
            (
                here.flags.contains(BlockFlags::LEGACY_SOLID),
                CellFlags::LEGACY_SOLID,
            ),
            (here.flags.contains(BlockFlags::WATER), CellFlags::WATER),
            (
                self.table.shape_without_context(here.class).is_none(),
                CellFlags::NO_COLLISION,
            ),
        ];
        let flags = marks
            .into_iter()
            .filter(|(set, _)| *set)
            .fold(CellFlags::NONE, |flags, (_, flag)| flags.union(flag));
        GroundCell {
            flags,
            malus: self.block_path_type(at, here, below).default_malus(),
            walk_value: if below.flags.contains(BlockFlags::GRASS_BLOCK) {
                GRASS_WALK_VALUE
            } else {
                OTHER_WALK_VALUE
            },
        }
    }

    fn line_of_sight(&self, from: DVec3, to: DVec3) -> bool {
        let start = DVec3::new(from.x, from.y, from.z);
        let end = DVec3::new(to.x, to.y, to.z);
        if (end - start).length_squared() > SIGHT_RANGE * SIGHT_RANGE {
            return false;
        }
        sight_clear(&SightBlocks(self.blocks), self.table, start, end)
    }
}

#[must_use]
pub(crate) fn path_cell_of(table: &CollisionTable, state: StateCollision) -> PathCell {
    let top = table
        .shape_without_context(state.class)
        .map_or(0.0, |shape| shape.top() as f32);
    let bits = CellBits::NONE
        .with(CellBits::AIR, state.flags.contains(BlockFlags::AIR))
        .with(CellBits::SHAPE, state.class != ShapeClass::Empty)
        .with(
            CellBits::SOLID,
            state.flags.contains(BlockFlags::LEGACY_SOLID),
        )
        .with(CellBits::FLOATABLE, state.flags.contains(BlockFlags::FLUID))
        .with(
            CellBits::CAULDRON,
            state.flags.contains(BlockFlags::CAULDRON),
        )
        .with(
            CellBits::DOOR_OR_FENCE,
            state.flags.intersects(
                BlockFlags::DOOR | BlockFlags::FENCE | BlockFlags::WALL | BlockFlags::FENCE_GATE,
            ),
        );
    PathCell {
        kind: state.path,
        top,
        bits,
    }
}

impl<V: BlockVolume> PathWorld for BlockGoalWorld<'_, V> {
    fn path_cell(&self, at: BlockPos) -> PathCell {
        path_cell_of(self.table, self.state(at))
    }

    fn min_y(&self) -> i32 {
        self.height.min
    }

    fn max_y(&self) -> i32 {
        self.height.max
    }

    fn column_loaded(&self, at: BlockPos) -> bool {
        self.blocks.section(SectionPos::from(at)).is_ok()
    }

    fn collides(&self, min: DVec3, max: DVec3) -> bool {
        let mob = MobPath::standing(
            DVec3::new((min.x + max.x) / 2.0, min.y, (min.z + max.z) / 2.0),
            (max.x - min.x) as f32,
            (max.y - min.y) as f32,
            0.0,
        );
        SearchView::new(self, &mob).collides(min, max)
    }
}

impl<'a, V: BlockVolume> PathSearchWorld for BlockGoalWorld<'a, V> {
    type Search<'s>
        = SearchView<'s, V>
    where
        Self: 's;

    fn open(&self, mob: &MobPath) -> Self::Search<'_> {
        SearchView::new(self, mob)
    }
}
