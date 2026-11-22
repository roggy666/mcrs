use glam::DVec3;
use mcrs_minecraft_path_type::PathType;
use mcrs_voxel::BlockPos;

use crate::NavFlags;
use crate::SearchWorld;
use crate::Searcher;
use crate::node::Node;
use crate::searcher::Scene;
use crate::world::floor_level_in;

const SIDES: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
const STEP_UP_LOW: f64 = 0.001;
const STEP_UP_HIGH: f64 = 0.002;
const NORMALIZE_LIMIT: f64 = 1.0e-5_f32 as f64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Candidate {
    None,
    Node(u32),
}

impl Candidate {
    fn index(self) -> Option<u32> {
        match self {
            Self::None => None,
            Self::Node(index) => Some(index),
        }
    }
}

pub(crate) fn start_node<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
) -> u32 {
    let mob = scene.mob;
    let block = scene.mob_block;
    let min_y = scene.world.min_y();
    let mut sy;
    if mob.can(NavFlags::CAN_FLOAT) && Searcher::cell(scene, block).floatable() {
        sy = block.y;
        while Searcher::cell(scene, BlockPos::new(block.x, sy + 1, block.z)).floatable() {
            sy += 1;
        }
    } else if mob.on_ground {
        sy = libm::floor(mob.position.y + 0.5) as i32;
    } else {
        sy = libm::floor(mob.position.y + 1.0) as i32;
        loop {
            let below = Searcher::cell(scene, BlockPos::new(block.x, sy - 1, block.z));
            if (!below.is_air() && !below.passable_on_land()) || sy <= min_y {
                break;
            }
            sy -= 1;
        }
    }
    let mut start = BlockPos::new(block.x, sy, block.z);
    if !can_start(searcher, scene, start) {
        let corners = [
            (mob.min.x, mob.min.z),
            (mob.min.x, mob.max.z),
            (mob.max.x, mob.min.z),
            (mob.max.x, mob.max.z),
        ];
        for (x, z) in corners {
            let corner = BlockPos::new(libm::floor(x) as i32, sy, libm::floor(z) as i32);
            if can_start(searcher, scene, corner) {
                start = corner;
                break;
            }
        }
    }
    let (kind, malus) = searcher.malus(scene, start);
    let index = searcher.node_at(start);
    if let Some(node) = searcher.node_mut(index) {
        node.kind = kind;
        node.malus = malus;
    }
    index
}

fn can_start<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
    at: BlockPos,
) -> bool {
    let (kind, malus) = searcher.malus(scene, at);
    kind != PathType::Open && malus >= 0.0
}

fn jump_size<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
    node: &Node,
) -> i32 {
    let above = searcher.malus(scene, node.pos.offset(0, 1, 0)).1;
    if above >= 0.0 && node.kind != PathType::Sticky {
        libm::floorf(scene.mob.step_height.max(1.0)) as i32
    } else {
        0
    }
}

fn level<W: SearchWorld + ?Sized>(scene: &mut Scene<'_, W>, at: BlockPos) -> f64 {
    floor_level_in(scene.world, at, scene.mob.can(NavFlags::CAN_FLOAT))
}

pub(crate) fn neighbours<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
    index: u32,
    out: &mut alloc::vec::Vec<u32>,
) {
    out.clear();
    let Some(node) = searcher.node(index).copied() else {
        return;
    };
    let jump = jump_size(searcher, scene, &node);
    let floor = level(scene, node.pos);
    let mut sides = [Candidate::None; 4];
    for (slot, (dx, dz)) in sides.iter_mut().zip(SIDES) {
        let column = BlockPos::new(node.pos.x + dx, node.pos.y, node.pos.z + dz);
        *slot = probe(searcher, scene, column, jump, floor, (dx, dz), node.kind);
        if let Some(candidate) = slot.index()
            && let Some(found) = searcher.node(candidate)
            && !found.closed
            && (found.malus >= 0.0 || node.malus < 0.0)
        {
            out.push(candidate);
        }
    }
    for first in 0..4 {
        let second = (first + 1) % 4;
        let (Some(a), Some(b)) = (sides.get(first).copied(), sides.get(second).copied()) else {
            continue;
        };
        if !diagonal_ok(searcher, scene, &node, a, b) {
            continue;
        }
        let (Some((ax, az)), Some((bx, bz))) = (SIDES.get(first), SIDES.get(second)) else {
            continue;
        };
        let column = BlockPos::new(node.pos.x + ax + bx, node.pos.y, node.pos.z + az + bz);
        let corner = probe(searcher, scene, column, jump, floor, (*ax, *az), node.kind);
        if let Some(candidate) = corner.index()
            && let Some(found) = searcher.node(candidate)
            && !found.closed
            && found.kind != PathType::WalkableDoor
            && found.malus >= 0.0
        {
            out.push(candidate);
        }
    }
}

fn diagonal_ok<W: SearchWorld + ?Sized>(
    searcher: &Searcher,
    scene: &mut Scene<'_, W>,
    node: &Node,
    a: Candidate,
    b: Candidate,
) -> bool {
    let (Some(a), Some(b)) = (a.index(), b.index()) else {
        return false;
    };
    let (Some(a), Some(b)) = (searcher.node(a), searcher.node(b)) else {
        return false;
    };
    if a.pos.y > node.pos.y || b.pos.y > node.pos.y {
        return false;
    }
    if a.kind == PathType::WalkableDoor || b.kind == PathType::WalkableDoor {
        return false;
    }
    let width = scene.mob.width;
    if width > 1.0 && (a.malus > 0.0 || b.malus > 0.0) {
        return false;
    }
    let both_fences = a.kind == PathType::Fence && b.kind == PathType::Fence && width < 0.5;
    [a, b]
        .iter()
        .all(|side| side.pos.y < node.pos.y || side.malus >= 0.0 || both_fences)
}

fn probe<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
    at: BlockPos,
    jump: i32,
    floor: f64,
    side: (i32, i32),
    from_kind: PathType,
) -> Candidate {
    if level(scene, at) - floor > scene.mob.jump_height() {
        return Candidate::None;
    }
    let (kind, malus) = searcher.malus(scene, at);
    let mut candidate = Candidate::None;
    if malus >= 0.0 {
        candidate = Candidate::Node(claim(searcher, at, kind, malus));
    }
    if from_kind.is_partial_collision()
        && let Candidate::Node(index) = candidate
        && searcher.node(index).is_some_and(|node| node.malus >= 0.0)
        && sweep_collides(searcher, scene, at)
    {
        candidate = Candidate::None;
    }
    if kind == PathType::Walkable {
        return candidate;
    }
    let negative = candidate
        .index()
        .and_then(|index| searcher.node(index))
        .is_none_or(|node| node.malus < 0.0);
    if negative
        && jump > 0
        && (kind != PathType::Fence || scene.mob.can(NavFlags::CAN_WALK_OVER_FENCES))
        && !matches!(
            kind,
            PathType::UnpassableRail | PathType::DropDown | PathType::PowderSnow
        )
    {
        return step_up(searcher, scene, at, jump, floor, side, from_kind);
    }
    if kind == PathType::Water && !scene.mob.can(NavFlags::CAN_FLOAT) {
        let mut y = at.y - 1;
        while y > scene.world.min_y() {
            let lower = BlockPos::new(at.x, y, at.z);
            let (lower_kind, lower_malus) = searcher.malus(scene, lower);
            if lower_kind != PathType::Water {
                break;
            }
            if lower_malus >= 0.0 {
                candidate = Candidate::Node(claim(searcher, lower, lower_kind, lower_malus));
            }
            y -= 1;
        }
        return candidate;
    }
    if kind == PathType::Open {
        return fall(searcher, scene, at);
    }
    if kind.is_partial_collision() && candidate == Candidate::None {
        let index = searcher.node_at(at);
        if let Some(node) = searcher.node_mut(index) {
            node.kind = kind;
            node.malus = kind.default_malus();
            node.closed = true;
        }
        return Candidate::Node(index);
    }
    candidate
}

fn claim(searcher: &mut Searcher, at: BlockPos, kind: PathType, malus: f32) -> u32 {
    let index = searcher.node_at(at);
    if let Some(node) = searcher.node_mut(index) {
        node.kind = kind;
        node.malus = node.malus.max(malus);
    }
    index
}

fn blocked(searcher: &mut Searcher, at: BlockPos) -> Candidate {
    let index = searcher.node_at(at);
    if let Some(node) = searcher.node_mut(index) {
        node.kind = PathType::Blocked;
        node.malus = -1.0;
    }
    Candidate::Node(index)
}

fn step_up<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
    at: BlockPos,
    jump: i32,
    floor: f64,
    side: (i32, i32),
    from_kind: PathType,
) -> Candidate {
    let above = at.offset(0, 1, 0);
    let result = probe(searcher, scene, above, jump - 1, floor, side, from_kind);
    let Candidate::Node(index) = result else {
        return Candidate::None;
    };
    let mob = scene.mob;
    if mob.width >= 1.0 {
        return result;
    }
    let Some(found) = searcher.node(index).copied() else {
        return Candidate::None;
    };
    if found.kind != PathType::Open && found.kind != PathType::Walkable {
        return result;
    }
    let cx = f64::from(at.x - side.0) + 0.5;
    let cz = f64::from(at.z - side.1) + 0.5;
    let half = f64::from(mob.width) / 2.0;
    let f1 = level(
        scene,
        BlockPos::from(DVec3::new(cx, f64::from(above.y), cz)),
    );
    let f2 = level(scene, found.pos);
    let min = DVec3::new(cx - half, f1 + STEP_UP_LOW, cz - half);
    let max = DVec3::new(
        cx + half,
        f64::from(mob.height) + f2 - STEP_UP_HIGH,
        cz + half,
    );
    if searcher.collides(scene, min, max) {
        return Candidate::None;
    }
    result
}

fn fall<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
    at: BlockPos,
) -> Candidate {
    let min_y = scene.world.min_y();
    let mut cy = at.y - 1;
    while cy >= min_y {
        if at.y - cy > scene.mob.fall_limit {
            return blocked(searcher, BlockPos::new(at.x, cy, at.z));
        }
        let lower = BlockPos::new(at.x, cy, at.z);
        let (kind, malus) = searcher.malus(scene, lower);
        if kind == PathType::Open {
            cy -= 1;
            continue;
        }
        if malus >= 0.0 {
            return Candidate::Node(claim(searcher, lower, kind, malus));
        }
        return blocked(searcher, lower);
    }
    blocked(searcher, at)
}

fn sweep_collides<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
    to: BlockPos,
) -> bool {
    let mob = scene.mob;
    let size = mob.max - mob.min;
    let target = DVec3::new(f64::from(to.x), f64::from(to.y), f64::from(to.z));
    let mut v = target - mob.position + size / 2.0;
    let mean = (size.x + size.y + size.z) / 3.0;
    let steps = libm::ceil(v.length() / mean) as i32;
    if steps <= 0 {
        return false;
    }
    v *= f64::from(1.0_f32 / steps as f32);
    let (mut min, mut max) = (mob.min, mob.max);
    for _ in 0..steps {
        min += v;
        max += v;
        if searcher.collides(scene, min, max) {
            return true;
        }
    }
    false
}

#[must_use]
pub(crate) fn normalized(v: DVec3) -> DVec3 {
    let length = v.length();
    if length < NORMALIZE_LIMIT {
        DVec3::ZERO
    } else {
        v / length
    }
}
