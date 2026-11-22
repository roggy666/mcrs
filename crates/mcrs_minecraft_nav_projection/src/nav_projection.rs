use std::collections::VecDeque;
use std::mem;

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct Versions {
    current: NavVersion,
    older: Vec<NavVersion>,
}

impl Versions {
    fn at(&self, tick: u64) -> Option<&NavSection> {
        if tick >= self.current.tick {
            return self.current.section.as_ref();
        }
        self.older
            .iter()
            .rev()
            .find(|version| version.tick <= tick)?
            .section
            .as_ref()
    }

    fn trim(&mut self, oldest: u64) {
        while let Some(first) = self.older.first() {
            let next = self
                .older
                .get(1)
                .map_or(self.current.tick, |next| next.tick);
            if first.tick <= oldest && next <= oldest {
                self.older.remove(0);
            } else {
                break;
            }
        }
    }

    fn is_gone(&self) -> bool {
        self.older.is_empty() && self.current.section.is_none()
    }

    fn len(&self) -> usize {
        self.older.len() + 1
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavProjection {
    sections: FxHashMap<SectionPos, Versions>,
    changed: VecDeque<(u64, SectionPos)>,
    table: NavTable,
    min_y: i32,
    max_y: i32,
}

impl NavProjection {
    #[must_use]
    pub fn new(table: NavTable, min_y: i32, max_y: i32) -> Self {
        Self {
            sections: FxHashMap::default(),
            changed: VecDeque::new(),
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
        self.changed.clear();
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
        match self.sections.get_mut(&pos) {
            Some(versions) if versions.current.tick == tick => versions.current.section = section,
            Some(versions) => {
                let previous = mem::replace(&mut versions.current, NavVersion { tick, section });
                versions.older.push(previous);
                self.changed.push_back((tick, pos));
            }
            None => {
                self.sections.insert(
                    pos,
                    Versions {
                        current: NavVersion { tick, section },
                        older: Vec::new(),
                    },
                );
                self.changed.push_back((tick, pos));
            }
        }
    }

    fn patch(&mut self, tick: u64, patch: NavPatch) {
        let pos = SectionPos::from(patch.pos);
        let Some(versions) = self.sections.get_mut(&pos) else {
            return;
        };
        let Some(current) = versions.current.section.as_ref() else {
            return;
        };
        let at = LocalPos::from(patch.pos);
        if current.get(at) == patch.nav {
            return;
        }
        if versions.current.tick == tick {
            if let Some(section) = versions.current.section.as_mut() {
                section.set(at, patch.nav);
            }
            return;
        }
        let mut copy = current.clone();
        copy.set(at, patch.nav);
        let previous = mem::replace(
            &mut versions.current,
            NavVersion {
                tick,
                section: Some(copy),
            },
        );
        versions.older.push(previous);
        self.changed.push_back((tick, pos));
    }

    pub fn retire(&mut self, oldest: u64) {
        while let Some(&(tick, pos)) = self.changed.front() {
            if tick > oldest {
                break;
            }
            self.changed.pop_front();
            let Some(versions) = self.sections.get_mut(&pos) else {
                continue;
            };
            versions.trim(oldest);
            if versions.is_gone() {
                self.sections.remove(&pos);
            }
        }
    }

    pub(crate) fn section_at(&self, at: SectionPos, tick: u64) -> Option<&NavSection> {
        self.sections.get(&at)?.at(tick)
    }

    #[must_use]
    pub fn sections(&self) -> usize {
        self.sections.len()
    }

    #[must_use]
    pub fn versions(&self) -> usize {
        self.sections.values().map(Versions::len).sum()
    }

    #[must_use]
    pub fn pending_retires(&self) -> usize {
        self.changed.len()
    }

    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        self.sections
            .values()
            .flat_map(|versions| core::iter::once(&versions.current).chain(versions.older.iter()))
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
        assert_eq!(projection.pending_retires(), 5);
        projection.retire(3);
        assert_eq!(projection.versions(), 3, "ticks 3, 4, 5 stay");
        assert_eq!(
            projection.pending_retires(),
            2,
            "only the sections that changed after 3 wait"
        );
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

    #[test]
    fn retire_touches_only_the_sections_that_changed() {
        let mut projection = projection();
        let quiet = SectionPos::new(9, 2, 9);
        projection.apply(
            1,
            NavDelta {
                landed: vec![section(here(), NavId(1)), section(quiet, NavId(2))],
                ..NavDelta::default()
            },
        );
        for tick in 2..=4u64 {
            projection.apply(
                tick,
                NavDelta {
                    patches: vec![NavPatch {
                        pos: BLOCK,
                        nav: NavId(tick as u16),
                    }],
                    ..NavDelta::default()
                },
            );
        }
        assert_eq!(
            projection.pending_retires(),
            5,
            "two landings and three patches"
        );
        projection.retire(4);
        assert_eq!(projection.pending_retires(), 0);
        assert_eq!(projection.versions(), 2, "one version each");
        assert_eq!(projection.at(4).lookup(BLOCK), NavLookup::Found(NavId(4)));
        assert!(projection.at(4).section(quiet).is_some());
    }

    #[test]
    fn a_new_table_forgets_every_section() {
        let mut projection = projection();
        projection.apply(
            1,
            NavDelta {
                landed: vec![section(here(), NavId(1))],
                ..NavDelta::default()
            },
        );
        projection.set_table(NavTable::default());
        assert_eq!(projection.sections(), 0);
        assert_eq!(projection.pending_retires(), 0);
    }
}
