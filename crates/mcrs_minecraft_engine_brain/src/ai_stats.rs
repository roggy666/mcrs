use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AiStats {
    pub mobs: usize,
    pub prepare: Duration,
    pub life_cycle: Duration,
    pub wait: Duration,
    pub tile_tasks: Duration,
    pub publish: Duration,
    pub stage: Duration,
    pub evaluations: u64,
    pub evaluations_total: u64,
    pub decisions: usize,
    pub applied: usize,
    pub waits: u64,
    pub lag_max: u64,
    pub lag_max_seen: u64,
    pub stage_lost: bool,
    pub without_tables: bool,
    pub blocks_sent: usize,
    pub projection_sections: usize,
    pub projection_bytes: usize,
}
