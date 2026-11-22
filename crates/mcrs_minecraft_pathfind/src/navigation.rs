use alloc::vec::Vec;

use glam::DVec3;
use mcrs_voxel::BlockPos;

use crate::FoundPath;
use crate::PathNode;
use crate::RequestKind;

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub nodes: Vec<PathNode>,
    pub next: usize,
    pub reached: bool,
}

impl Path {
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.next >= self.nodes.len()
    }

    #[must_use]
    pub fn next_node(&self) -> Option<&PathNode> {
        self.nodes.get(self.next)
    }

    #[must_use]
    pub fn after_next(&self) -> Option<&PathNode> {
        self.nodes.get(self.next + 1)
    }

    #[must_use]
    pub fn end(&self) -> Option<&PathNode> {
        self.nodes.last()
    }

    pub fn advance(&mut self) {
        self.next = self.next.saturating_add(1);
    }

    #[must_use]
    pub fn same_positions(&self, other: &[PathNode]) -> bool {
        self.nodes.len() == other.len()
            && self
                .nodes
                .iter()
                .zip(other)
                .all(|(a, b)| a.pos() == b.pos())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathBox {
    pub min: BlockPos,
    pub max: BlockPos,
}

impl PathBox {
    #[must_use]
    pub fn of(nodes: &[PathNode]) -> Option<Self> {
        let first = nodes.first()?;
        let mut min = first.pos();
        let mut max = first.pos();
        for node in nodes {
            min = BlockPos::new(min.x.min(node.x), min.y.min(node.y), min.z.min(node.z));
            max = BlockPos::new(max.x.max(node.x), max.y.max(node.y), max.z.max(node.z));
        }
        Some(Self { min, max })
    }

    #[must_use]
    pub fn touches(&self, at: BlockPos, margin: i32) -> bool {
        at.x >= self.min.x - margin
            && at.x <= self.max.x + margin
            && at.y >= self.min.y - margin
            && at.y <= self.max.y + margin
            && at.z >= self.min.z - margin
            && at.z <= self.max.z + margin
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StuckWindow {
    pub tick: u64,
    pub position: DVec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Timeout {
    pub node: Option<BlockPos>,
    pub time: u64,
    pub timer: u64,
    pub limit: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acceptance {
    Changed,
    Same,
    Dropped,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Navigation {
    pub path: Option<Path>,
    pub target: Option<BlockPos>,
    pub reach: i32,
    pub speed_modifier: f32,
    pub ticks: u64,
    pub stuck: StuckWindow,
    pub timeout: Timeout,
    pub last_recompute: u64,
    pub recompute_waiting: bool,
    pub bounds: Option<PathBox>,
    pub stuck_end: bool,
}

impl Navigation {
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.path.as_ref().is_some_and(|path| !path.is_done())
    }

    #[must_use]
    pub fn has_target(&self, target: BlockPos) -> bool {
        self.is_running() && self.target == Some(target)
    }

    pub fn drop_path(&mut self) {
        self.path = None;
        self.bounds = None;
        self.recompute_waiting = false;
    }

    pub fn reset_stuck(&mut self, position: DVec3) {
        self.stuck = StuckWindow {
            tick: self.ticks,
            position,
        };
    }

    pub fn reset_timeout(&mut self) {
        self.timeout = Timeout::default();
    }

    pub fn accept(
        &mut self,
        kind: RequestKind,
        result: Option<FoundPath>,
        speed_modifier: f32,
        position: DVec3,
        is_cauldron: impl Fn(BlockPos) -> bool,
    ) -> Acceptance {
        let Some(found) = result else {
            if kind == RequestKind::Entity {
                return Acceptance::Same;
            }
            self.drop_path();
            return Acceptance::Dropped;
        };
        let mut nodes = found.nodes;
        cauldron_rule(&mut nodes, &is_cauldron);
        if nodes.is_empty() {
            self.drop_path();
            return Acceptance::Dropped;
        }
        let changed = match &self.path {
            Some(current) if !current.is_done() => !current.same_positions(&nodes),
            _ => true,
        };
        if changed {
            self.bounds = PathBox::of(&nodes);
            self.path = Some(Path {
                nodes,
                next: 0,
                reached: found.reached,
            });
            self.reset_stuck(position);
            self.reset_timeout();
            self.stuck_end = false;
        } else if let Some(current) = self.path.as_mut() {
            current.reached = found.reached;
        }
        self.target = Some(found.target);
        self.reach = found.reach;
        self.speed_modifier = speed_modifier;
        if changed {
            Acceptance::Changed
        } else {
            Acceptance::Same
        }
    }

    pub fn keep_current(&mut self, speed_modifier: f32) {
        self.speed_modifier = speed_modifier;
    }
}

fn cauldron_rule(nodes: &mut [PathNode], is_cauldron: &impl Fn(BlockPos) -> bool) {
    for index in 0..nodes.len() {
        let Some(node) = nodes.get(index).copied() else {
            break;
        };
        if !is_cauldron(node.pos()) {
            continue;
        }
        let old_y = node.y;
        if let Some(slot) = nodes.get_mut(index) {
            slot.y += 1;
        }
        if let Some(next) = nodes.get_mut(index + 1)
            && old_y >= next.y
        {
            next.y = old_y + 1;
        }
    }
}
