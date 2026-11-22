use glam::DVec3;
use mcrs_voxel::BlockPos;

use crate::BuildHeight;
use crate::GroundCell;

pub trait GoalWorld {
    fn build_height(&self) -> BuildHeight;

    fn cell(&self, at: BlockPos) -> GroundCell;

    fn line_of_sight(&self, from: DVec3, to: DVec3) -> bool;
}
