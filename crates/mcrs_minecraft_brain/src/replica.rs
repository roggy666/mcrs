use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::mem;

use crate::Fact;
use crate::MobView;
use crate::PlayerTable;
use crate::TileIndex;
use crate::TileViews;
use mcrs_minecraft_entity_key::TileKey;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Replica {
    tiles: BTreeMap<TileKey, VecDeque<TileVersion>>,
    players: VecDeque<PlayersVersion>,
    facts: VecDeque<TickFacts>,
    spare: Vec<Vec<MobView>>,
}

#[derive(Clone, Debug, PartialEq)]
struct TileVersion {
    tick: u64,
    views: Vec<MobView>,
    index: TileIndex,
}

#[derive(Clone, Debug, PartialEq)]
struct PlayersVersion {
    tick: u64,
    table: PlayerTable,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TickFacts {
    tick: u64,
    tiles: BTreeMap<TileKey, Vec<Fact>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TileSnapshot {
    pub views: Vec<MobView>,
    pub index: TileIndex,
}

impl TileSnapshot {
    #[must_use]
    pub fn new(tile: TileKey, mut views: Vec<MobView>) -> Self {
        views.sort_by_key(|view| view.key);
        let index = TileIndex::build(tile, &views);
        Self { views, index }
    }
}

impl Replica {
    pub fn publish_tile(&mut self, tick: u64, tile: TileKey, snapshot: TileSnapshot) {
        let versions = self.tiles.entry(tile).or_default();
        if let Some(last) = versions.back_mut()
            && last.tick == tick
        {
            last.views = snapshot.views;
            last.index = snapshot.index;
            return;
        }
        versions.push_back(TileVersion {
            tick,
            views: snapshot.views,
            index: snapshot.index,
        });
    }

    pub fn publish_views(&mut self, tick: u64, tile: TileKey, views: &[MobView]) {
        let mut buffer = self.spare.pop().unwrap_or_default();
        buffer.clear();
        buffer.extend_from_slice(views);
        let index = TileIndex::build(tile, &buffer);
        let versions = self.tiles.entry(tile).or_default();
        if let Some(last) = versions.back_mut()
            && last.tick == tick
        {
            let old = mem::replace(&mut last.views, buffer);
            self.spare.push(old);
            last.index = index;
            return;
        }
        versions.push_back(TileVersion {
            tick,
            views: buffer,
            index,
        });
    }

    pub fn publish_players(&mut self, tick: u64, table: PlayerTable) {
        if let Some(last) = self.players.back_mut()
            && last.tick == tick
        {
            last.table = table;
            return;
        }
        self.players.push_back(PlayersVersion { tick, table });
    }

    pub fn publish_facts(
        &mut self,
        tick: u64,
        tile: TileKey,
        facts: impl IntoIterator<Item = Fact>,
    ) {
        let at = match self.facts.back() {
            Some(last) if last.tick == tick => self.facts.len() - 1,
            _ => {
                self.facts.push_back(TickFacts {
                    tick,
                    tiles: BTreeMap::new(),
                });
                self.facts.len() - 1
            }
        };
        if let Some(entry) = self.facts.get_mut(at) {
            entry.tiles.entry(tile).or_default().extend(facts);
        }
    }

    #[must_use]
    pub fn tile(&self, tile: TileKey, tick: u64) -> Option<TileViews<'_>> {
        let version = self
            .tiles
            .get(&tile)?
            .iter()
            .rev()
            .find(|version| version.tick <= tick)?;
        Some(TileViews {
            tile,
            views: &version.views,
            index: &version.index,
        })
    }

    #[must_use]
    pub fn views(&self, tile: TileKey, tick: u64) -> &[MobView] {
        self.tile(tile, tick).map_or(&[], |found| found.views)
    }

    #[must_use]
    pub fn players(&self, tick: u64) -> Option<&PlayerTable> {
        self.players
            .iter()
            .rev()
            .find(|version| version.tick <= tick)
            .map(|version| &version.table)
    }

    #[must_use]
    pub fn facts(&self, tile: TileKey, tick: u64) -> &[Fact] {
        self.facts
            .iter()
            .find(|entry| entry.tick == tick)
            .and_then(|entry| entry.tiles.get(&tile))
            .map_or(&[], Vec::as_slice)
    }

    pub fn retire(&mut self, oldest: u64) {
        let spare = &mut self.spare;
        self.tiles.retain(|_, versions| {
            while versions.len() > 1 && versions.get(1).is_some_and(|next| next.tick <= oldest) {
                if let Some(old) = versions.pop_front() {
                    spare.push(old.views);
                }
            }
            versions
                .back()
                .is_some_and(|version| !version.views.is_empty())
                || versions.len() > 1
        });
        spare.truncate(SPARE_LIMIT);
        retire_versions(&mut self.players, oldest, |version| version.tick);
        while self.facts.front().is_some_and(|entry| entry.tick < oldest) {
            self.facts.pop_front();
        }
    }
}

const SPARE_LIMIT: usize = 4096;

fn retire_versions<T>(versions: &mut VecDeque<T>, oldest: u64, tick: impl Fn(&T) -> u64) {
    while versions.len() > 1 && versions.get(1).is_some_and(|next| tick(next) <= oldest) {
        versions.pop_front();
    }
}
