use mcrs_minecraft_nav_projection::NavSection;
use mcrs_minecraft_nav_projection::NavTable;
use mcrs_minecraft_nav_projection::NavView;
use mcrs_minecraft_physics::CollisionSection;
use mcrs_minecraft_physics::CollisionWorld;
use mcrs_minecraft_physics::SectionLookup;
use mcrs_minecraft_physics::SectionMasks;
use mcrs_minecraft_physics::WorldBorder;
use mcrs_voxel::BlockStateId;
use mcrs_voxel::LocalPos;
use mcrs_voxel::SectionPos;

#[derive(Clone, Copy, Debug)]
pub struct NavSight<'a> {
    view: NavView<'a>,
}

impl<'a> NavSight<'a> {
    #[must_use]
    pub const fn new(view: NavView<'a>) -> Self {
        Self { view }
    }
}

pub struct NavCollisionSection<'a> {
    section: &'a NavSection,
    table: &'a NavTable,
}

impl CollisionSection for NavCollisionSection<'_> {
    fn masks(&self) -> Option<&SectionMasks> {
        None
    }

    fn state(&self, at: LocalPos) -> BlockStateId {
        self.table.representative(self.section.get(at))
    }
}

impl<'a> CollisionWorld for NavSight<'a> {
    type Section<'s>
        = NavCollisionSection<'a>
    where
        Self: 's;

    fn section(&self, at: SectionPos) -> SectionLookup<Self::Section<'_>> {
        let lowest = at.y * SectionPos::SIZE as i32;
        if lowest + SectionPos::SIZE as i32 <= self.view.min_y() || lowest > self.view.max_y() {
            return SectionLookup::Outside;
        }
        match self.view.section(at) {
            Some(section) => SectionLookup::Loaded(NavCollisionSection {
                section,
                table: self.view.table(),
            }),
            None => SectionLookup::Unloaded,
        }
    }

    fn border(&self) -> Option<&WorldBorder> {
        None
    }
}
