use mcrs_minecraft_brain::Fact;
use mcrs_minecraft_brain::PlayerTable;
use mcrs_minecraft_brain::TileSnapshot;
use mcrs_minecraft_chunk::ChunkBlocks;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_minecraft_nav_projection::NavPatch;
use mcrs_voxel::SectionPos;

use crate::StageTables;

#[derive(Debug)]
pub struct Arrival<B> {
    pub id: EntityId,
    pub key: EntityUuid,
    pub tile: TileKey,
    pub phase: u8,
    pub brain: B,
}

#[derive(Clone, Debug)]
pub struct Landed {
    pub pos: SectionPos,
    pub blocks: ChunkBlocks,
}

#[derive(Clone, Debug, Default)]
pub struct BlockDelta {
    pub landed: Vec<Landed>,
    pub unloaded: Vec<SectionPos>,
    pub patches: Vec<NavPatch>,
}

impl BlockDelta {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.landed.is_empty() && self.unloaded.is_empty() && self.patches.is_empty()
    }

    pub fn clear(&mut self) {
        self.landed.clear();
        self.unloaded.clear();
        self.patches.clear();
    }
}

#[derive(Debug)]
pub struct TickInput<B> {
    pub tick: u64,
    pub arrivals: Vec<Arrival<B>>,
    pub departures: Vec<EntityId>,
    pub moves: Vec<(EntityId, TileKey)>,
    pub tiles: Vec<(TileKey, TileSnapshot)>,
    pub facts: Vec<(TileKey, Fact)>,
    pub players: PlayerTable,
    pub blocks: BlockDelta,
}

impl<B> TickInput<B> {
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            arrivals: Vec::new(),
            departures: Vec::new(),
            moves: Vec::new(),
            tiles: Vec::new(),
            facts: Vec::new(),
            players: PlayerTable::default(),
            blocks: BlockDelta::default(),
        }
    }
}

#[derive(Debug)]
pub enum AiInput<B> {
    Tables(Box<StageTables>),
    Tick(Box<TickInput<B>>),
    #[cfg(feature = "testing")]
    Slow(core::time::Duration),
}
