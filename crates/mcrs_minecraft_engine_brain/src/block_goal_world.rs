use glam::DVec3;
use mcrs_minecraft_brain_stage::NavRows;
use mcrs_minecraft_brain_stage::feet_top;
use mcrs_minecraft_brain_stage::ground_cell;
use mcrs_minecraft_brain_stage::path_cell_of;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_minecraft_goal_world::GroundCell;
use mcrs_minecraft_nav_projection::NavRow;
use mcrs_minecraft_pathfind::MobPath;
use mcrs_minecraft_pathfind::PathCell;
use mcrs_minecraft_pathfind::PathSearchWorld;
use mcrs_minecraft_pathfind::PathWorld;
use mcrs_minecraft_physics::BlockFlags;
use mcrs_minecraft_physics::CollisionTable;
use mcrs_minecraft_physics::Tables;
use mcrs_minecraft_physics::sight_clear;
use mcrs_minecraft_volume::BlockVolume;
use mcrs_minecraft_volume::Unavailable;
use mcrs_voxel::BlockPos;
use mcrs_voxel::SectionPos;

use crate::SearchView;
use crate::sight_blocks::SightBlocks;

const SIGHT_RANGE: f64 = 128.0;

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
        path_cell_of(self.table, NavRow::UNLOADED)
    }

    #[must_use]
    pub fn is_cauldron(&self, at: BlockPos) -> bool {
        self.row(at).flags.contains(BlockFlags::CAULDRON)
    }

    #[must_use]
    pub fn feet_top(&self, at: BlockPos) -> Option<f64> {
        feet_top(self.table, self.row(at), at.y)
    }
}

impl<V: BlockVolume> NavRows for BlockGoalWorld<'_, V> {
    fn row(&self, at: BlockPos) -> NavRow {
        match self
            .blocks
            .block(mcrs_voxel::BlockPos::new(at.x, at.y, at.z))
        {
            Ok(state) => NavRow::of(&self.table.state(state)),
            Err(Unavailable::Outside) => NavRow::AIR,
            Err(Unavailable::Unloaded | Unavailable::Foreign) => NavRow::UNLOADED,
        }
    }

    fn lowest_y(&self) -> i32 {
        self.height.min
    }
}

impl<V: BlockVolume> GoalWorld for BlockGoalWorld<'_, V> {
    fn build_height(&self) -> BuildHeight {
        self.height
    }

    fn cell(&self, at: BlockPos) -> GroundCell {
        ground_cell(self, self.table, at)
    }

    fn line_of_sight(&self, from: DVec3, to: DVec3) -> bool {
        if (to - from).length_squared() > SIGHT_RANGE * SIGHT_RANGE {
            return false;
        }
        sight_clear(&SightBlocks(self.blocks), self.table, from, to)
    }
}

impl<V: BlockVolume> PathWorld for BlockGoalWorld<'_, V> {
    fn path_cell(&self, at: BlockPos) -> PathCell {
        path_cell_of(self.table, self.row(at))
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
