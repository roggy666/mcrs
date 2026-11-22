use std::time::Duration;

use mcrs_minecraft_brain::Fact;
use mcrs_minecraft_brain::IssuedDecisions;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::TileKey;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RunStats {
    pub stage: Duration,
    pub evaluations: u64,
    pub pops: u64,
    pub deferred: usize,
    pub slices: usize,
    pub decisions: usize,
    pub mobs: usize,
    pub sections: usize,
    pub projection_bytes: usize,
    pub without_tables: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Recycled {
    pub views: Vec<MobView>,
    pub facts: Vec<(TileKey, Fact)>,
    pub moves: Vec<(EntityId, TileKey)>,
    pub departures: Vec<EntityId>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AiOutput {
    pub tick: u64,
    pub decisions: IssuedDecisions,
    pub run: RunStats,
    pub recycled: Recycled,
}
