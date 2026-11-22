use std::time::Duration;

use mcrs_minecraft_brain::IssuedDecisions;

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
pub struct AiOutput {
    pub tick: u64,
    pub decisions: IssuedDecisions,
    pub run: RunStats,
}
