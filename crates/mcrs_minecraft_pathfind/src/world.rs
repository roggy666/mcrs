use glam::DVec3;
use mcrs_voxel::BlockPos;

use crate::PathCell;

pub trait PathWorld {
    fn cell(&self, at: BlockPos) -> PathCell;

    fn min_y(&self) -> i32;

    fn max_y(&self) -> i32;

    fn column_loaded(&self, x: i32, z: i32) -> bool;

    fn collides(&self, min: DVec3, max: DVec3) -> bool;
}

#[must_use]
pub fn floor_level<W: PathWorld + ?Sized>(world: &W, at: BlockPos, can_float: bool) -> f64 {
    if can_float && world.cell(at).floatable() {
        return f64::from(at.y) + 0.5;
    }
    let below = world.cell(at.below());
    f64::from(at.y - 1) + f64::from(below.top)
}
