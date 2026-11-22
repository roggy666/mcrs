use mcrs_voxel::BlockPos;
use mcrs_voxel::LocalPos;
use mcrs_voxel::SectionPos;

use crate::NavId;
use crate::NavProjection;
use crate::NavRow;
use crate::NavSection;
use crate::NavTable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavLookup {
    Found(NavId),
    Unloaded,
    Outside,
}

#[derive(Clone, Copy, Debug)]
pub struct NavView<'a> {
    projection: &'a NavProjection,
    tick: u64,
}

impl<'a> NavView<'a> {
    pub(crate) const fn new(projection: &'a NavProjection, tick: u64) -> Self {
        Self { projection, tick }
    }

    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    #[must_use]
    pub const fn at(&self, tick: u64) -> Self {
        Self::new(self.projection, tick)
    }

    #[must_use]
    pub const fn is_live(&self) -> bool {
        self.tick >= self.projection.floor()
    }

    #[must_use]
    pub const fn table(&self) -> &'a NavTable {
        self.projection.table()
    }

    #[must_use]
    pub const fn min_y(&self) -> i32 {
        self.projection.min_y()
    }

    #[must_use]
    pub const fn max_y(&self) -> i32 {
        self.projection.max_y()
    }

    #[must_use]
    pub fn section(&self, at: SectionPos) -> Option<&'a NavSection> {
        self.projection.section_at(at, self.tick)
    }

    #[must_use]
    pub fn lookup(&self, at: BlockPos) -> NavLookup {
        if at.y < self.min_y() || at.y > self.max_y() {
            return NavLookup::Outside;
        }
        match self.section(SectionPos::from(at)) {
            Some(section) => NavLookup::Found(section.get(LocalPos::from(at))),
            None => NavLookup::Unloaded,
        }
    }

    #[must_use]
    pub fn row(&self, at: BlockPos) -> NavRow {
        match self.lookup(at) {
            NavLookup::Found(nav) => self.table().row(nav),
            NavLookup::Unloaded => NavRow::UNLOADED,
            NavLookup::Outside => NavRow::AIR,
        }
    }

    #[must_use]
    pub fn is_loaded(&self, at: BlockPos) -> bool {
        matches!(self.lookup(at), NavLookup::Found(_))
    }
}
