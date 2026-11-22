use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::mem;

use hashbrown::HashMap;

use crate::Cadence;
use crate::Decision;
use crate::Intents;
use crate::IssuedDecisions;
use crate::ItemTables;
use crate::KindTable;
use crate::MobBrain;
use crate::MobView;
use crate::Neighbours;
use crate::PlayerRow;
use crate::PlayerTable;
use crate::Replica;
use crate::StageConfig;
use crate::TileBrains;
use crate::Turn;
use crate::TurnScratch;
use crate::WakeMode;
use crate::sort_paths;
use crate::tile_brains::Deferred;
use crate::tile_brains::RangeOut;
use crate::tile_brains::Removed;
use glam::DVec3;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::TileKey;

#[derive(Clone, Debug)]
pub struct Stage<B> {
    config: StageConfig,
    tiles: BTreeMap<TileKey, TileBrains<B>>,
    locations: HashMap<EntityId, Location>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    tile: TileKey,
    key: EntityUuid,
}

#[derive(Debug)]
pub struct TickContext<'a, W: ?Sized> {
    pub tick: u64,
    pub config: StageConfig,
    pub replica: &'a Replica,
    pub items: &'a ItemTables,
    pub kinds: &'a KindTable,
    pub world: &'a W,
}

#[derive(Debug)]
pub struct TileWork<'a, B> {
    tile: TileKey,
    budget: u32,
    range: TileRange<'a, B>,
}

#[derive(Debug)]
struct TileRange<'a, B> {
    first: usize,
    ids: &'a [EntityId],
    keys: &'a [EntityUuid],
    cadences: &'a [Cadence],
    wakes: &'a mut [u64],
    brains: &'a mut [B],
    out: &'a mut RangeOut,
}

const SEARCH_COST: usize = 16;
const MIN_TILE_BUDGET: u32 = 560;
const RESERVE_SHARE: u32 = 4;

impl<B> Stage<B> {
    #[must_use]
    pub fn new(config: StageConfig) -> Self {
        Self {
            config,
            tiles: BTreeMap::new(),
            locations: HashMap::new(),
        }
    }

    #[must_use]
    pub const fn config(&self) -> &StageConfig {
        &self.config
    }

    pub fn spawn(
        &mut self,
        tile: TileKey,
        mob: EntityId,
        key: EntityUuid,
        cadence: Cadence,
        brain: B,
    ) {
        self.despawn(mob);
        let wake = cadence.next_evaluation(0);
        self.tiles
            .entry(tile)
            .or_default()
            .insert(mob, key, cadence, wake, brain);
        self.locations.insert(mob, Location { tile, key });
    }

    pub fn despawn(&mut self, mob: EntityId) -> Option<B> {
        let location = self.locations.remove(&mob)?;
        let tiles = self.tiles.get_mut(&location.tile)?;
        let removed = tiles.remove(&location.key);
        if tiles.is_empty() {
            self.tiles.remove(&location.tile);
        }
        removed.map(|removed| removed.brain)
    }

    pub fn relocate(&mut self, mob: EntityId, to: TileKey) {
        let Some(location) = self.locations.get_mut(&mob) else {
            return;
        };
        if location.tile == to {
            return;
        }
        let from = mem::replace(&mut location.tile, to);
        let key = location.key;
        let Some(tiles) = self.tiles.get_mut(&from) else {
            return;
        };
        let removed = tiles.remove(&key);
        if tiles.is_empty() {
            self.tiles.remove(&from);
        }
        if let Some(removed) = removed {
            self.tiles.entry(to).or_default().insert(
                removed.mob,
                removed.key,
                removed.cadence,
                removed.wake,
                removed.brain,
            );
        }
    }

    pub fn relocate_all(&mut self, moves: impl IntoIterator<Item = (EntityId, TileKey)>) {
        let mut leaving: BTreeMap<TileKey, Vec<EntityUuid>> = BTreeMap::new();
        let mut targets: BTreeMap<EntityUuid, TileKey> = BTreeMap::new();
        for (mob, to) in moves {
            let Some(location) = self.locations.get_mut(&mob) else {
                continue;
            };
            if location.tile == to {
                continue;
            }
            let from = mem::replace(&mut location.tile, to);
            leaving.entry(from).or_default().push(location.key);
            targets.insert(location.key, to);
        }
        let mut entering: BTreeMap<TileKey, Vec<Removed<B>>> = BTreeMap::new();
        for (from, mut keys) in leaving {
            let Some(tiles) = self.tiles.get_mut(&from) else {
                continue;
            };
            keys.sort_unstable();
            for removed in tiles.remove_all(&keys) {
                if let Some(&to) = targets.get(&removed.key) {
                    entering.entry(to).or_default().push(removed);
                }
            }
            if tiles.is_empty() {
                self.tiles.remove(&from);
            }
        }
        for (to, mut entries) in entering {
            entries.sort_by_key(|entry| entry.key);
            self.tiles.entry(to).or_default().insert_all(entries);
        }
    }

    #[must_use]
    pub fn tile_of(&self, mob: EntityId) -> Option<TileKey> {
        self.locations.get(&mob).map(|location| location.tile)
    }

    #[must_use]
    pub fn key_of(&self, mob: EntityId) -> Option<EntityUuid> {
        self.locations.get(&mob).map(|location| location.key)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.locations.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    pub fn tiles(&self) -> impl Iterator<Item = (TileKey, &TileBrains<B>)> {
        self.tiles.iter().map(|(&tile, brains)| (tile, brains))
    }

    #[must_use]
    pub fn brain(&self, mob: EntityId) -> Option<&B> {
        let location = self.locations.get(&mob)?;
        self.tiles.get(&location.tile)?.brain(&location.key)
    }

    #[must_use]
    pub fn evaluations(&self) -> u64 {
        self.tiles.values().map(TileBrains::evaluations).sum()
    }

    #[must_use]
    pub fn pops(&self) -> u64 {
        self.tiles.values().map(TileBrains::pops).sum()
    }

    #[must_use]
    pub fn deferred(&self) -> usize {
        self.tiles.values().map(TileBrains::deferred).sum()
    }

    #[must_use]
    pub fn slices(&self) -> usize {
        self.tiles
            .values()
            .filter(|brains| brains.has_open_slice())
            .count()
    }

    pub fn work(&mut self) -> Vec<TileWork<'_, B>> {
        let (dimension, tile) = (self.config.path_budget(), self.config.tile_path_budget());
        self.work_budgeted(dimension, tile)
    }

    pub fn work_budgeted(&mut self, dimension_budget: u32, tile_cap: u32) -> Vec<TileWork<'_, B>> {
        let mut order: Vec<(u64, TileKey)> = self
            .tiles
            .iter()
            .map(|(&tile, brains)| (brains.oldest_deferral().unwrap_or(u64::MAX), tile))
            .collect();
        order.sort_unstable();
        let tiles = u32::try_from(self.tiles.len().max(1)).unwrap_or(u32::MAX);
        let base = (dimension_budget / tiles).clamp(MIN_TILE_BUDGET.min(tile_cap), tile_cap);
        let mut reserve = dimension_budget / RESERVE_SHARE;
        let mut budgets: BTreeMap<TileKey, u32> = BTreeMap::new();
        for (oldest, tile) in order {
            let mut given = base;
            if oldest != u64::MAX {
                let extra = tile_cap.saturating_sub(base).min(reserve);
                reserve = reserve.saturating_sub(extra);
                given = given.saturating_add(extra);
            }
            budgets.insert(tile, given);
        }
        let split = self.config.tile_split().max(1);
        let mut work = Vec::new();
        for (&tile, brains) in &mut self.tiles {
            let budget = budgets.get(&tile).copied().unwrap_or(0);
            split_work(tile, brains, budget, split, &mut work);
        }
        work
    }

    pub fn collect(&mut self, tick: u64) -> IssuedDecisions {
        let mut issued = IssuedDecisions {
            tick,
            tiles: BTreeMap::new(),
            paths: Vec::new(),
        };
        for (&tile, brains) in &mut self.tiles {
            let mut decisions = Vec::new();
            for out in &mut brains.outs {
                if decisions.is_empty() {
                    decisions = mem::take(&mut out.decisions);
                } else {
                    decisions.append(&mut out.decisions);
                }
                issued.paths.append(&mut out.paths);
                brains.deferred.append(&mut out.deferred);
                if out.slice.is_some() {
                    brains.slice = out.slice.take();
                }
                brains.evaluations = brains.evaluations.saturating_add(out.evaluations);
                brains.pops = brains.pops.saturating_add(out.pops);
                out.evaluations = 0;
                out.pops = 0;
            }
            if !decisions.is_empty() {
                issued.tiles.insert(tile, decisions);
            }
        }
        sort_paths(&mut issued.paths);
        issued
    }
}

impl<B> TileWork<'_, B> {
    #[must_use]
    pub const fn tile(&self) -> TileKey {
        self.tile
    }

    #[must_use]
    pub fn cost(&self) -> usize {
        self.range
            .ids
            .len()
            .saturating_add(self.range.out.pending().saturating_mul(SEARCH_COST))
    }

    pub fn run<W: ?Sized>(&mut self, cx: &TickContext<'_, W>, scratch: &mut TurnScratch)
    where
        B: MobBrain<W>,
    {
        let tick = cx.tick;
        let range = &mut self.range;
        scratch.budget = self.budget;
        for fact in cx.replica.facts(self.tile, tick) {
            if let Ok(at) = range.keys.binary_search(&fact.key)
                && let (Some(wake), Some(cadence)) =
                    (range.wakes.get_mut(at), range.cadences.get(at))
            {
                *wake = (*wake).min(cadence.next_evaluation(tick));
            }
        }
        let views = cx.replica.views(self.tile, tick);
        let neighbours = Neighbours::new(self.tile, |tile| cx.replica.tile(tile, tick));
        let table = cx.replica.players(tick);
        let empty = PlayerTable::default();
        let table = table.unwrap_or(&empty);
        let mut loaded_range: Option<f64> = None;
        let deferred = take_deferred(range.out, range.keys, scratch);
        let RangeOut {
            near_players,
            decisions,
            paths,
            deferred: still_deferred,
            slice,
            evaluations,
            pops,
        } = &mut *range.out;
        let mut turns = Turns {
            cx,
            views,
            neighbours: &neighbours,
            table,
            near_players,
            loaded_range: &mut loaded_range,
        };
        retry_deferred(
            deferred,
            &mut turns,
            scratch,
            tick,
            range.first,
            Retrying {
                ids: range.ids,
                keys: range.keys,
                cadences: range.cadences,
                states: range.brains,
                paths,
                still_deferred,
                decisions,
            },
        );
        let rows = range
            .ids
            .iter()
            .zip(range.keys.iter())
            .zip(range.cadences.iter());
        for (index, (((mob, key), cadence), (wake, brain))) in rows
            .zip(range.wakes.iter_mut().zip(range.brains.iter_mut()))
            .enumerate()
        {
            if *wake > tick {
                continue;
            }
            if !cadence.is_full_evaluation(tick) {
                *wake = cadence.next_evaluation(tick);
                continue;
            }
            let Some(turn) = turns.turn(range.first + index, *mob, key, *cadence, brain) else {
                *wake = cadence.next_evaluation(tick.saturating_add(1));
                continue;
            };
            let mut intents = Intents::default();
            scratch.clear();
            let next = brain.evaluate(&turn, scratch, &mut intents);
            *evaluations += 1;
            *wake = match cx.config.wake() {
                WakeMode::Sleep => next.max(tick.saturating_add(1)),
                WakeMode::Poll => turn.next_evaluation(),
            };
            paths.append(&mut scratch.paths);
            if scratch.deferred {
                still_deferred.push(Deferred { tick, key: *key });
            }
            if !intents.is_empty() {
                decisions.push(Decision {
                    mob: *mob,
                    key: *key,
                    issue_tick: tick,
                    intents,
                });
            }
        }
        *slice = scratch.slice.take();
        *pops += u64::from(self.budget.saturating_sub(scratch.budget));
    }
}

fn split_work<'a, B>(
    tile: TileKey,
    brains: &'a mut TileBrains<B>,
    budget: u32,
    split: usize,
    work: &mut Vec<TileWork<'a, B>>,
) {
    let TileBrains {
        ids,
        keys,
        cadences,
        wakes,
        brains: states,
        outs,
        deferred,
        slice,
        ..
    } = brains;
    let len = ids.len();
    if len == 0 {
        return;
    }
    let parts = len.div_ceil(split).max(1);
    let chunk = len.div_ceil(parts).max(1);
    let parts = len.div_ceil(chunk);
    outs.resize_with(parts, RangeOut::default);
    outs.truncate(parts);
    for out in outs.iter_mut() {
        out.clear();
    }
    let mut routed = mem::take(deferred);
    routed.sort_unstable();
    for retry in routed {
        if let Ok(at) = keys.binary_search(&retry.key)
            && let Some(out) = outs.get_mut(at / chunk)
        {
            out.deferred.push(retry);
        }
    }
    if let Some(open) = slice.take()
        && let Ok(at) = keys.binary_search(&open.key)
        && let Some(out) = outs.get_mut(at / chunk)
    {
        out.slice = Some(open);
    }
    let shares = share_budget(budget, outs);
    let rows = ids
        .chunks(chunk)
        .zip(keys.chunks(chunk))
        .zip(cadences.chunks(chunk))
        .zip(wakes.chunks_mut(chunk))
        .zip(states.chunks_mut(chunk))
        .zip(outs.iter_mut())
        .zip(shares);
    for (index, ((((((ids, keys), cadences), wakes), brains), out), share)) in rows.enumerate() {
        work.push(TileWork {
            tile,
            budget: share,
            range: TileRange {
                first: index * chunk,
                ids,
                keys,
                cadences,
                wakes,
                brains,
                out,
            },
        });
    }
}

fn share_budget(budget: u32, outs: &[RangeOut]) -> Vec<u32> {
    let parts = u32::try_from(outs.len().max(1)).unwrap_or(u32::MAX);
    let pending: Vec<u64> = outs
        .iter()
        .map(|out| u64::try_from(out.pending()).unwrap_or(u64::MAX))
        .collect();
    let total: u64 = pending.iter().sum();
    let mut shares: Vec<u32> = if total == 0 {
        pending.iter().map(|_| budget / parts).collect()
    } else {
        pending
            .iter()
            .map(|count| u32::try_from(u64::from(budget) * count / total).unwrap_or(u32::MAX))
            .collect()
    };
    let given: u32 = shares
        .iter()
        .fold(0, |sum, share| sum.saturating_add(*share));
    if let Some(first) = shares.first_mut() {
        *first = first.saturating_add(budget.saturating_sub(given));
    }
    shares
}

fn take_deferred(
    out: &mut RangeOut,
    keys: &[EntityUuid],
    scratch: &mut TurnScratch,
) -> Vec<Deferred> {
    let mut deferred = mem::take(&mut out.deferred);
    deferred.sort_unstable();
    scratch.slice = out.slice.take();
    if scratch
        .slice
        .as_deref()
        .is_some_and(|slice| keys.binary_search(&slice.key).is_err())
    {
        scratch.slice = None;
    }
    if let Some(slice) = scratch.slice.as_deref()
        && let Some(at) = deferred.iter().position(|retry| retry.key == slice.key)
    {
        let first = deferred.remove(at);
        deferred.insert(0, first);
    }
    deferred
}

struct Retrying<'r, B> {
    ids: &'r [EntityId],
    keys: &'r [EntityUuid],
    cadences: &'r [Cadence],
    states: &'r mut [B],
    paths: &'r mut Vec<crate::IssuedPath>,
    still_deferred: &'r mut Vec<Deferred>,
    decisions: &'r mut Vec<Decision>,
}

fn retry_deferred<B, W: ?Sized>(
    deferred: Vec<Deferred>,
    turns: &mut Turns<'_, '_, W>,
    scratch: &mut TurnScratch,
    tick: u64,
    first: usize,
    retrying: Retrying<'_, B>,
) where
    B: MobBrain<W>,
{
    let Retrying {
        ids,
        keys,
        cadences,
        states,
        paths,
        still_deferred,
        decisions,
    } = retrying;
    for retry in deferred {
        let Some(index) = keys.binary_search(&retry.key).ok() else {
            continue;
        };
        let (Some(mob), Some(cadence), Some(brain)) =
            (ids.get(index), cadences.get(index), states.get_mut(index))
        else {
            continue;
        };
        let Some(turn) = turns.turn(first + index, *mob, &retry.key, *cadence, brain) else {
            still_deferred.push(retry);
            continue;
        };
        let mut intents = Intents::default();
        scratch.clear();
        brain.retry_search(&turn, scratch, &mut intents);
        paths.append(&mut scratch.paths);
        if scratch.deferred {
            still_deferred.push(retry);
        }
        if !intents.is_empty() {
            decisions.push(Decision {
                mob: *mob,
                key: retry.key,
                issue_tick: tick,
                intents,
            });
        }
    }
}

struct Turns<'t, 'a, W: ?Sized> {
    cx: &'t TickContext<'a, W>,
    views: &'t [MobView],
    neighbours: &'t Neighbours<'t>,
    table: &'t PlayerTable,
    near_players: &'t mut Vec<PlayerRow>,
    loaded_range: &'t mut Option<f64>,
}

impl<'t, 'a, W: ?Sized> Turns<'t, 'a, W> {
    fn turn<B: MobBrain<W>>(
        &mut self,
        index: usize,
        mob: EntityId,
        key: &EntityUuid,
        cadence: Cadence,
        brain: &B,
    ) -> Option<Turn<'_, W>> {
        let view = view_of(self.views, index, mob, key)?;
        let range = brain.rules().player_range;
        if self.loaded_range.is_none_or(|loaded| loaded < range) {
            *self.loaded_range = Some(range);
            self.near_players.clear();
            load_players(self.table, self.views, range, self.near_players);
        }
        Some(Turn {
            tick: self.cx.tick,
            seed: self.cx.config.seed(),
            max_lag: self.cx.config.max_lag(),
            cadence,
            view,
            players: self.near_players,
            table: self.table,
            neighbours: self.neighbours,
            items: self.cx.items,
            kinds: self.cx.kinds,
            world: self.cx.world,
        })
    }
}

fn view_of<'a>(
    views: &'a [MobView],
    index: usize,
    mob: EntityId,
    key: &EntityUuid,
) -> Option<&'a MobView> {
    if let Some(view) = views.get(index)
        && view.mob == mob
    {
        return Some(view);
    }
    let at = views.binary_search_by_key(key, |view| view.key).ok()?;
    views.get(at).filter(|view| view.mob == mob)
}

fn load_players(table: &PlayerTable, views: &[MobView], range: f64, out: &mut Vec<PlayerRow>) {
    if table.is_empty() {
        return;
    }
    let mut min = DVec3::new(f64::INFINITY, 0.0, f64::INFINITY);
    let mut max = DVec3::new(f64::NEG_INFINITY, 0.0, f64::NEG_INFINITY);
    for view in views {
        min.x = min.x.min(view.position.x);
        min.z = min.z.min(view.position.z);
        max.x = max.x.max(view.position.x);
        max.z = max.z.max(view.position.z);
    }
    if min.x > max.x {
        return;
    }
    let margin = range + 1.0;
    table.collect_in(
        DVec3::new(min.x - margin, 0.0, min.z - margin),
        DVec3::new(max.x + margin, 0.0, max.z + margin),
        out,
    );
}
