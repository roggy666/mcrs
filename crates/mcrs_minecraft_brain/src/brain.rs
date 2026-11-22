use alloc::vec::Vec;

use crate::BrainRules;
use crate::Cadence;
use crate::Candidate;
use crate::Intents;
use crate::IssuedPath;
use crate::ItemTables;
use crate::KeyedRandom;
use crate::KindTable;
use crate::MobView;
use crate::Neighbours;
use crate::Part;
use crate::PlayerRow;
use crate::PlayerTable;
use crate::WorldSeed;
use mcrs_minecraft_pathfind::Searcher;

pub trait MobBrain<W: ?Sized> {
    fn rules(&self) -> &'static BrainRules;

    fn evaluate(
        &mut self,
        turn: &Turn<'_, W>,
        scratch: &mut TurnScratch,
        intents: &mut Intents,
    ) -> u64;

    fn retry_search(
        &mut self,
        _turn: &Turn<'_, W>,
        _scratch: &mut TurnScratch,
        _intents: &mut Intents,
    ) {
    }
}

#[derive(Debug)]
pub struct Turn<'a, W: ?Sized> {
    pub tick: u64,
    pub seed: WorldSeed,
    pub max_lag: u8,
    pub cadence: Cadence,
    pub view: &'a MobView,
    pub players: &'a [PlayerRow],
    pub table: &'a PlayerTable,
    pub neighbours: &'a Neighbours<'a>,
    pub items: &'a ItemTables,
    pub kinds: &'a KindTable,
    pub world: &'a W,
}

impl<W: ?Sized> Turn<'_, W> {
    #[must_use]
    pub fn random(&self, part: Part) -> KeyedRandom {
        KeyedRandom::new(self.seed, &self.view.key, self.tick, part)
    }

    #[must_use]
    pub fn next_evaluation(&self) -> u64 {
        self.cadence.next_evaluation(self.tick.saturating_add(1))
    }
}

#[derive(Debug)]
pub struct TurnScratch {
    pub candidates: Vec<Candidate>,
    pub ranked: Vec<(f64, Candidate)>,
    pub searcher: Searcher,
    pub budget: u32,
    pub paths: Vec<IssuedPath>,
    pub deferred: bool,
}

impl Default for TurnScratch {
    fn default() -> Self {
        Self {
            candidates: Vec::new(),
            ranked: Vec::new(),
            searcher: Searcher::default(),
            budget: u32::MAX,
            paths: Vec::new(),
            deferred: false,
        }
    }
}

impl TurnScratch {
    pub fn clear(&mut self) {
        self.candidates.clear();
        self.ranked.clear();
        self.deferred = false;
    }
}
