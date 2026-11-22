use glam::DVec3;
use mcrs_voxel::BlockPos;

use crate::MobPath;
use crate::PathCell;

pub trait PathWorld {
    fn path_cell(&self, at: BlockPos) -> PathCell;

    fn min_y(&self) -> i32;

    fn max_y(&self) -> i32;

    fn column_loaded(&self, at: BlockPos) -> bool;

    fn collides(&self, min: DVec3, max: DVec3) -> bool;
}

#[must_use]
pub fn floor_level<W: PathWorld + ?Sized>(world: &W, at: BlockPos, can_float: bool) -> f64 {
    if can_float && world.path_cell(at).floatable() {
        return f64::from(at.y) + 0.5;
    }
    let below = world.path_cell(at.below());
    f64::from(at.y - 1) + f64::from(below.top)
}

impl<W: PathWorld + ?Sized> PathWorld for &W {
    fn path_cell(&self, at: BlockPos) -> PathCell {
        (**self).path_cell(at)
    }

    fn min_y(&self) -> i32 {
        (**self).min_y()
    }

    fn max_y(&self) -> i32 {
        (**self).max_y()
    }

    fn column_loaded(&self, at: BlockPos) -> bool {
        (**self).column_loaded(at)
    }

    fn collides(&self, min: DVec3, max: DVec3) -> bool {
        (**self).collides(min, max)
    }
}

pub trait PathSearchWorld: PathWorld {
    type Search<'a>: PathWorld
    where
        Self: 'a;

    fn open(&self, mob: &MobPath) -> Self::Search<'_>;
}
