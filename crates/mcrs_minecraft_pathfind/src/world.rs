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

    fn clear_between(&self, from: DVec3, to: DVec3) -> bool;
}

pub trait SearchWorld {
    fn path_cell(&mut self, at: BlockPos) -> PathCell;

    fn min_y(&mut self) -> i32;

    fn max_y(&mut self) -> i32;

    fn column_loaded(&mut self, at: BlockPos) -> bool;

    fn collides(&mut self, min: DVec3, max: DVec3) -> bool;
}

#[must_use]
pub fn fluid_free<W: PathWorld + ?Sized>(world: &W, from: DVec3, to: DVec3) -> bool {
    let steps = libm::ceil((to - from).length() * 4.0).max(1.0) as u32;
    (0..=steps).all(|step| {
        let point = from + (to - from) * (f64::from(step) / f64::from(steps));
        !world.path_cell(BlockPos::from(point)).floatable()
    })
}

#[must_use]
pub fn floor_level<W: PathWorld + ?Sized>(world: &W, at: BlockPos, can_float: bool) -> f64 {
    if can_float && world.path_cell(at).floatable() {
        return f64::from(at.y) + 0.5;
    }
    let below = world.path_cell(at.below());
    f64::from(at.y - 1) + f64::from(below.top)
}

#[must_use]
pub(crate) fn floor_level_in<W: SearchWorld + ?Sized>(
    world: &mut W,
    at: BlockPos,
    can_float: bool,
) -> f64 {
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

    fn clear_between(&self, from: DVec3, to: DVec3) -> bool {
        (**self).clear_between(from, to)
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

impl<W: PathWorld + ?Sized> SearchWorld for &W {
    fn path_cell(&mut self, at: BlockPos) -> PathCell {
        (**self).path_cell(at)
    }

    fn min_y(&mut self) -> i32 {
        (**self).min_y()
    }

    fn max_y(&mut self) -> i32 {
        (**self).max_y()
    }

    fn column_loaded(&mut self, at: BlockPos) -> bool {
        (**self).column_loaded(at)
    }

    fn collides(&mut self, min: DVec3, max: DVec3) -> bool {
        (**self).collides(min, max)
    }
}

pub trait PathSearchWorld: PathWorld {
    type Search<'a>: SearchWorld
    where
        Self: 'a;

    fn open(&self, mob: &MobPath) -> Self::Search<'_>;

    fn version(&self) -> u64 {
        0
    }

    fn reopen(&self, mob: &MobPath, version: u64) -> Option<Self::Search<'_>> {
        (version == self.version()).then(|| self.open(mob))
    }
}
