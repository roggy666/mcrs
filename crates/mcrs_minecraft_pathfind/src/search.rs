use alloc::vec::Vec;

use mcrs_voxel::BlockPos;

use crate::MobPath;
use crate::PathNode;
use crate::PathWorld;
use crate::Region;
use crate::Request;
use crate::Searcher;
use crate::ground;
use crate::node::distance;
use crate::searcher::Scene;
use crate::searcher::Target;

const HEURISTIC_WEIGHT: f32 = 1.5;

#[derive(Clone, Debug, PartialEq)]
pub struct FoundPath {
    pub nodes: Vec<PathNode>,
    pub target: BlockPos,
    pub reach: i32,
    pub reached: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchOutcome {
    pub path: Option<FoundPath>,
    pub pops: u32,
}

pub fn search<W: PathWorld + ?Sized>(
    world: &W,
    mob: &MobPath,
    request: &Request,
    searcher: &mut Searcher,
) -> SearchOutcome {
    searcher.clear();
    let mob_block = mob.block_pos();
    let scene = Scene {
        world,
        region: Region::around(mob_block, request.length),
        mob,
        mob_block,
    };
    let budget = request.node_budget();
    searcher
        .targets
        .extend(request.targets.iter().map(|&pos| Target {
            pos,
            best: None,
            best_value: f32::MAX,
            reached: false,
        }));
    if searcher.targets.is_empty() {
        return SearchOutcome {
            path: None,
            pops: 0,
        };
    }
    let start = ground::start_node(searcher, &scene);
    let start_pos = searcher.node(start).map_or(mob_block, |node| node.pos);
    let h = heuristic(searcher, start);
    if let Some(node) = searcher.node_mut(start) {
        node.g = 0.0;
        node.h = h;
        node.f = h;
    }
    let mut nodes = searcher.nodes.clone();
    core::mem::swap(&mut nodes, &mut searcher.nodes);
    searcher.heap.insert(&mut searcher.nodes, start);
    let mut pops: u32 = 0;
    let mut neighbours: Vec<u32> = Vec::new();
    while !searcher.heap.is_empty() {
        pops = pops.saturating_add(1);
        if pops >= budget {
            break;
        }
        let Some(current) = searcher.heap.pop(&mut searcher.nodes) else {
            break;
        };
        if let Some(node) = searcher.node_mut(current) {
            node.closed = true;
        }
        let Some(node) = searcher.node(current).copied() else {
            break;
        };
        let mut reached = false;
        for target in &mut searcher.targets {
            if node.manhattan_to(target.pos) <= request.reach {
                target.reached = true;
                reached = true;
            }
        }
        if reached {
            break;
        }
        if node.distance_to(start_pos) >= request.length {
            continue;
        }
        ground::neighbours(searcher, &scene, current, &mut neighbours);
        for &next in &neighbours {
            let Some(q) = searcher.node(next).copied() else {
                continue;
            };
            let e = distance(node.pos, q.pos);
            let walked = node.walked + e;
            let t = node.g + e + q.malus;
            if let Some(slot) = searcher.node_mut(next) {
                slot.walked = walked;
            }
            let in_open = q.heap.is_some();
            if walked < request.length && (!in_open || t < q.g) {
                let h = heuristic(searcher, next) * HEURISTIC_WEIGHT;
                let f = t + h;
                if let Some(slot) = searcher.node_mut(next) {
                    slot.previous = Some(current);
                    slot.g = t;
                    slot.h = h;
                }
                if in_open {
                    searcher.heap.change_key(&mut searcher.nodes, next, f);
                } else {
                    if let Some(slot) = searcher.node_mut(next) {
                        slot.f = f;
                    }
                    searcher.heap.insert(&mut searcher.nodes, next);
                }
            }
        }
    }
    let path = choose(searcher, request.reach);
    SearchOutcome { path, pops }
}

fn heuristic(searcher: &mut Searcher, index: u32) -> f32 {
    let Some(node) = searcher.node(index).copied() else {
        return f32::MAX;
    };
    let mut best = f32::MAX;
    for target in &mut searcher.targets {
        let value = node.distance_to(target.pos);
        if value < target.best_value {
            target.best_value = value;
            target.best = Some(index);
        }
        best = best.min(value);
    }
    best
}

fn trail(searcher: &Searcher, end: u32) -> Vec<PathNode> {
    let mut nodes = Vec::new();
    let mut cursor = Some(end);
    while let Some(index) = cursor {
        let Some(node) = searcher.node(index) else {
            break;
        };
        nodes.push(PathNode::new(node.pos, node.kind));
        cursor = node.previous;
    }
    nodes.reverse();
    nodes
}

fn choose(searcher: &Searcher, reach: i32) -> Option<FoundPath> {
    let reached: Vec<&Target> = searcher
        .targets
        .iter()
        .filter(|target| target.reached)
        .collect();
    if !reached.is_empty() {
        let mut best: Option<(usize, FoundPath)> = None;
        for target in reached {
            let Some(end) = target.best else {
                continue;
            };
            let nodes = trail(searcher, end);
            let count = nodes.len();
            if best.as_ref().is_none_or(|(len, _)| count < *len) {
                best = Some((
                    count,
                    FoundPath {
                        nodes,
                        target: target.pos,
                        reach,
                        reached: true,
                    },
                ));
            }
        }
        return best.map(|(_, path)| path);
    }
    let mut best: Option<((i32, usize), FoundPath)> = None;
    for target in &searcher.targets {
        let Some(end) = target.best else {
            continue;
        };
        let Some(node) = searcher.node(end) else {
            continue;
        };
        let nodes = trail(searcher, end);
        let rank = (node.manhattan_to(target.pos), nodes.len());
        if best.as_ref().is_none_or(|(held, _)| rank < *held) {
            best = Some((
                rank,
                FoundPath {
                    nodes,
                    target: target.pos,
                    reach,
                    reached: false,
                },
            ));
        }
    }
    best.map(|(_, path)| path)
}
