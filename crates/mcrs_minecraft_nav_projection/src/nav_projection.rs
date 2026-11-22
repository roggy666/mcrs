use std::collections::VecDeque;

use mcrs_voxel::BlockPos;
use mcrs_voxel::LocalPos;
use mcrs_voxel::SectionPos;
use rustc_hash::FxHashMap;

use crate::NavId;
use crate::NavSection;
use crate::NavTable;
use crate::NavView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavPatch {
    pub pos: BlockPos,
    pub nav: NavId,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavDelta {
    pub landed: Vec<(SectionPos, NavSection)>,
    pub unloaded: Vec<SectionPos>,
    pub patches: Vec<NavPatch>,
}

impl NavDelta {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.landed.is_empty() && self.unloaded.is_empty() && self.patches.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NavVersion {
    tick: u64,
    section: Option<NavSection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavProjection {
    sections: FxHashMap<SectionPos, VecDeque<NavVersion>>,
    table: NavTable,
    min_y: i32,
    max_y: i32,
}

impl NavProjection {
    #[must_use]
    pub fn new(table: NavTable, min_y: i32, max_y: i32) -> Self {
        Self {
            sections: FxHashMap::default(),
            table,
            min_y,
            max_y,
        }
    }

    #[must_use]
    pub const fn table(&self) -> &NavTable {
        &self.table
    }

    #[must_use]
    pub const fn min_y(&self) -> i32 {
        self.min_y
    }

    #[must_use]
    pub const fn max_y(&self) -> i32 {
        self.max_y
    }

    pub fn set_table(&mut self, table: NavTable) {
        self.table = table;
        self.sections.clear();
    }

    #[must_use]
    pub const fn at(&self, tick: u64) -> NavView<'_> {
        NavView::new(self, tick)
    }

    pub fn apply(&mut self, tick: u64, delta: NavDelta) {
        for (pos, section) in delta.landed {
            self.publish(tick, pos, Some(section));
        }
        for pos in delta.unloaded {
            if self.sections.contains_key(&pos) {
                self.publish(tick, pos, None);
            }
        }
        for patch in delta.patches {
            self.patch(tick, patch);
        }
    }

    fn publish(&mut self, tick: u64, pos: SectionPos, section: Option<NavSection>) {
        let versions = self.sections.entry(pos).or_default();
        if let Some(last) = versions.back_mut()
            && last.tick == tick
        {
            last.section = section;
            return;
        }
        versions.push_back(NavVersion { tick, section });
    }

    fn patch(&mut self, tick: u64, patch: NavPatch) {
        let Some(versions) = self.sections.get_mut(&SectionPos::from(patch.pos)) else {
            return;
        };
        let Some(last) = versions.back() else {
            return;
        };
        let Some(current) = last.section.as_ref() else {
            return;
        };
        let at = LocalPos::from(patch.pos);
        if current.get(at) == patch.nav {
            return;
        }
        if last.tick != tick {
            let mut copy = current.clone();
            copy.set(at, patch.nav);
            versions.push_back(NavVersion {
                tick,
                section: Some(copy),
            });
            return;
        }
        if let Some(last) = versions.back_mut()
            && let Some(section) = last.section.as_mut()
        {
            section.set(at, patch.nav);
        }
    }

    pub fn retire(&mut self, oldest: u64) {
        self.sections.retain(|_, versions| {
            while versions.len() > 1 && versions.get(1).is_some_and(|next| next.tick <= oldest) {
                versions.pop_front();
            }
            versions.len() > 1 || versions.back().is_some_and(|last| last.section.is_some())
        });
    }

    pub(crate) fn section_at(&self, at: SectionPos, tick: u64) -> Option<&NavSection> {
        self.sections
            .get(&at)?
            .iter()
            .rev()
            .find(|version| version.tick <= tick)?
            .section
            .as_ref()
    }

    #[must_use]
    pub fn sections(&self) -> usize {
        self.sections.len()
    }

    #[must_use]
    pub fn versions(&self) -> usize {
        self.sections.values().map(VecDeque::len).sum()
    }

    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        self.sections
            .values()
            .flat_map(|versions| versions.iter())
            .map(|version| {
                size_of::<NavVersion>() + version.section.as_ref().map_or(0, NavSection::heap_bytes)
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NavLookup;

    fn projection() -> NavProjection {
        NavProjection::new(NavTable::default(), -64, 319)
    }

    fn section(pos: SectionPos, nav: NavId) -> (SectionPos, NavSection) {
        (pos, NavSection::Uniform(nav))
    }

    const BLOCK: BlockPos = BlockPos::new(17, 33, 49);

    fn here() -> SectionPos {
        SectionPos::new(1, 2, 3)
    }

    #[test]
    fn a_change_of_tick_t_plus_one_is_seen_by_t_plus_one_and_not_by_t() {
        let mut projection = projection();
        projection.apply(
            10,
            NavDelta {
                landed: vec![section(here(), NavId(5))],
                ..NavDelta::default()
            },
        );
        projection.apply(
            11,
            NavDelta {
                patches: vec![NavPatch {
                    pos: BLOCK,
                    nav: NavId(7),
                }],
                ..NavDelta::default()
            },
        );
        assert_eq!(projection.at(10).lookup(BLOCK), NavLookup::Found(NavId(5)));
        assert_eq!(projection.at(11).lookup(BLOCK), NavLookup::Found(NavId(7)));
        assert_eq!(projection.at(12).lookup(BLOCK), NavLookup::Found(NavId(7)));
        assert_eq!(projection.at(9).lookup(BLOCK), NavLookup::Unloaded);
        assert_eq!(projection.versions(), 2);
        assert!(
            projection
                .at(10)
                .section(here())
                .is_some_and(NavSection::is_uniform),
            "the old version keeps its uniform section"
        );
    }

    #[test]
    fn patches_of_one_tick_share_one_copy_and_a_same_value_patch_makes_none() {
        let mut projection = projection();
        projection.apply(
            1,
            NavDelta {
                landed: vec![section(here(), NavId(5))],
                ..NavDelta::default()
            },
        );
        projection.apply(
            2,
            NavDelta {
                patches: vec![
                    NavPatch {
                        pos: BLOCK,
                        nav: NavId(6),
                    },
                    NavPatch {
                        pos: BlockPos::new(18, 33, 49),
                        nav: NavId(8),
                    },
                ],
                ..NavDelta::default()
            },
        );
        assert_eq!(projection.versions(), 2);
        projection.apply(
            3,
            NavDelta {
                patches: vec![NavPatch {
                    pos: BLOCK,
                    nav: NavId(6),
                }],
                ..NavDelta::default()
            },
        );
        assert_eq!(projection.versions(), 2, "the same value is no change");
        assert_eq!(
            projection.at(3).lookup(BlockPos::new(18, 33, 49)),
            NavLookup::Found(NavId(8))
        );
    }

    #[test]
    fn an_unload_keeps_the_old_version_readable_until_retired() {
        let mut projection = projection();
        projection.apply(
            1,
            NavDelta {
                landed: vec![section(here(), NavId(5))],
                ..NavDelta::default()
            },
        );
        projection.apply(
            2,
            NavDelta {
                unloaded: vec![here()],
                ..NavDelta::default()
            },
        );
        assert_eq!(projection.at(1).lookup(BLOCK), NavLookup::Found(NavId(5)));
        assert_eq!(projection.at(2).lookup(BLOCK), NavLookup::Unloaded);
        projection.retire(1);
        assert_eq!(projection.at(1).lookup(BLOCK), NavLookup::Found(NavId(5)));
        projection.retire(2);
        assert_eq!(
            projection.sections(),
            0,
            "an unloaded section with no older version leaves"
        );
        projection.apply(
            3,
            NavDelta {
                patches: vec![NavPatch {
                    pos: BLOCK,
                    nav: NavId(9),
                }],
                ..NavDelta::default()
            },
        );
        assert_eq!(
            projection.sections(),
            0,
            "a patch of an unknown section is ignored"
        );
    }

    #[test]
    fn retire_keeps_the_version_that_the_oldest_live_tick_reads() {
        let mut projection = projection();
        for tick in 1..=5u64 {
            projection.apply(
                tick,
                NavDelta {
                    patches: vec![NavPatch {
                        pos: BLOCK,
                        nav: NavId(tick as u16),
                    }],
                    landed: if tick == 1 {
                        vec![section(here(), NavId(0))]
                    } else {
                        Vec::new()
                    },
                    ..NavDelta::default()
                },
            );
        }
        assert_eq!(projection.versions(), 5);
        projection.retire(3);
        assert_eq!(projection.versions(), 3, "ticks 3, 4, 5 stay");
        assert_eq!(projection.at(3).lookup(BLOCK), NavLookup::Found(NavId(3)));
        assert_eq!(
            projection.at(2).lookup(BLOCK),
            NavLookup::Unloaded,
            "a read older than the kept versions is unloaded, as in the replica"
        );
    }

    #[test]
    fn outside_the_build_height_reads_as_air_and_a_missing_section_as_unloaded() {
        let projection = projection();
        let view = projection.at(1);
        assert_eq!(view.lookup(BlockPos::new(0, 400, 0)), NavLookup::Outside);
        assert_eq!(view.row(BlockPos::new(0, 400, 0)), crate::NavRow::AIR);
        assert_eq!(view.lookup(BLOCK), NavLookup::Unloaded);
        assert_eq!(view.row(BLOCK), crate::NavRow::UNLOADED);
        assert!(!view.is_loaded(BLOCK));
    }
}
