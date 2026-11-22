use alloc::vec::Vec;

use glam::DVec3;
use hashbrown::HashMap;
use mcrs_minecraft_path_type::PathType;
use mcrs_voxel::BlockPos;

use crate::MobPath;
use crate::NavFlags;
use crate::PathCell;
use crate::PathWorld;
use crate::Region;
use crate::heap::Heap;
use crate::node::Node;

#[derive(Debug, Default)]
pub struct Searcher {
    pub(crate) nodes: Vec<Node>,
    pub(crate) index: HashMap<(i32, i32, i32), u32>,
    pub(crate) heap: Heap,
    pub(crate) block_types: HashMap<(i32, i32, i32), PathType>,
    pub(crate) mob_types: HashMap<(i32, i32, i32), PathType>,
    pub(crate) boxes: HashMap<[u64; 6], bool>,
    pub(crate) targets: Vec<Target>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Target {
    pub(crate) pos: BlockPos,
    pub(crate) best: Option<u32>,
    pub(crate) best_value: f32,
    pub(crate) reached: bool,
}

pub(crate) struct Scene<'a, W: ?Sized> {
    pub(crate) world: &'a W,
    pub(crate) region: Region,
    pub(crate) mob: &'a MobPath,
    pub(crate) mob_block: BlockPos,
}

fn key(pos: BlockPos) -> (i32, i32, i32) {
    (pos.x, pos.y, pos.z)
}

impl Searcher {
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.index.clear();
        self.heap.clear();
        self.block_types.clear();
        self.mob_types.clear();
        self.boxes.clear();
        self.targets.clear();
    }

    pub(crate) fn node_at(&mut self, pos: BlockPos) -> u32 {
        if let Some(&index) = self.index.get(&key(pos)) {
            return index;
        }
        let index = u32::try_from(self.nodes.len()).unwrap_or(u32::MAX);
        self.nodes.push(Node::new(pos));
        self.index.insert(key(pos), index);
        index
    }

    pub(crate) fn node(&self, index: u32) -> Option<&Node> {
        self.nodes.get(index as usize)
    }

    pub(crate) fn node_mut(&mut self, index: u32) -> Option<&mut Node> {
        self.nodes.get_mut(index as usize)
    }

    pub(crate) fn cell<W: PathWorld + ?Sized>(scene: &Scene<'_, W>, at: BlockPos) -> PathCell {
        if !scene.region.contains(at.x, at.z)
            || at.y < scene.world.min_y()
            || at.y > scene.world.max_y()
            || !scene.world.column_loaded(at)
        {
            return PathCell::AIR;
        }
        scene.world.path_cell(at)
    }

    pub(crate) fn block_type<W: PathWorld + ?Sized>(
        &mut self,
        scene: &Scene<'_, W>,
        at: BlockPos,
    ) -> PathType {
        if let Some(&kind) = self.block_types.get(&key(at)) {
            return kind;
        }
        let kind = block_path_type(scene, at);
        self.block_types.insert(key(at), kind);
        kind
    }

    pub(crate) fn mob_type<W: PathWorld + ?Sized>(
        &mut self,
        scene: &Scene<'_, W>,
        at: BlockPos,
    ) -> PathType {
        if let Some(&kind) = self.mob_types.get(&key(at)) {
            return kind;
        }
        let kind = self.mob_path_type(scene, at);
        self.mob_types.insert(key(at), kind);
        kind
    }

    pub(crate) fn malus<W: PathWorld + ?Sized>(
        &mut self,
        scene: &Scene<'_, W>,
        at: BlockPos,
    ) -> (PathType, f32) {
        let kind = self.mob_type(scene, at);
        (kind, scene.mob.malus.get(kind))
    }

    pub(crate) fn collides<W: PathWorld + ?Sized>(
        &mut self,
        scene: &Scene<'_, W>,
        min: DVec3,
        max: DVec3,
    ) -> bool {
        let key = [
            min.x.to_bits(),
            min.y.to_bits(),
            min.z.to_bits(),
            max.x.to_bits(),
            max.y.to_bits(),
            max.z.to_bits(),
        ];
        if let Some(&hit) = self.boxes.get(&key) {
            return hit;
        }
        let hit = scene.world.collides(min, max);
        self.boxes.insert(key, hit);
        hit
    }

    fn mob_path_type<W: PathWorld + ?Sized>(
        &mut self,
        scene: &Scene<'_, W>,
        at: BlockPos,
    ) -> PathType {
        let mob = scene.mob;
        let wide = mob.cells_wide();
        let high = mob.cells_high();
        let rail_near = self.block_type(scene, scene.mob_block) == PathType::Rail
            || self.block_type(scene, scene.mob_block.below()) == PathType::Rail;
        let mut set: u32 = 0;
        for i in 0..wide {
            for j in 0..high {
                for k in 0..wide {
                    let mut kind = self.block_type(scene, at.offset(i, j, k));
                    kind = match kind {
                        PathType::DoorWoodClosed
                            if mob.can(NavFlags::CAN_OPEN_DOORS)
                                && mob.can(NavFlags::CAN_PASS_DOORS) =>
                        {
                            PathType::WalkableDoor
                        }
                        PathType::DoorOpen if !mob.can(NavFlags::CAN_PASS_DOORS) => {
                            PathType::Blocked
                        }
                        PathType::Rail if !rail_near => PathType::UnpassableRail,
                        other => other,
                    };
                    set |= 1 << kind.number();
                }
            }
        }
        if set.count_ones() == 1
            && let Some(kind) = PathType::from_number(set.trailing_zeros() as u8)
        {
            return kind;
        }
        if set & (1 << PathType::Fence.number()) != 0 {
            return PathType::Fence;
        }
        if set & (1 << PathType::UnpassableRail.number()) != 0 {
            return PathType::UnpassableRail;
        }
        let mut highest = PathType::Blocked;
        let mut highest_malus = mob.malus.get(PathType::Blocked);
        for number in 0..PathType::COUNT as u8 {
            if set & (1 << number) == 0 {
                continue;
            }
            let Some(kind) = PathType::from_number(number) else {
                continue;
            };
            let malus = mob.malus.get(kind);
            if malus < 0.0 {
                return kind;
            }
            if malus >= highest_malus {
                highest = kind;
                highest_malus = malus;
            }
        }
        let centre = self.block_type(scene, at);
        if wide > 1 {
            if mob.malus.get(centre) < highest_malus
                && mob.malus.get(PathType::BigMobsCloseToDanger) < highest_malus
            {
                return PathType::BigMobsCloseToDanger;
            }
            return highest;
        }
        if centre == PathType::Open && highest != PathType::Open && highest_malus == 0.0 {
            return PathType::Open;
        }
        highest
    }
}

pub(crate) fn block_path_type<W: PathWorld + ?Sized>(
    scene: &Scene<'_, W>,
    at: BlockPos,
) -> PathType {
    let here = Searcher::cell(scene, at).kind;
    if here != PathType::Open || at.y <= scene.world.min_y() {
        return here;
    }
    let below = Searcher::cell(scene, at.below()).kind;
    if let Some(kind) = PathType::over(below) {
        return kind;
    }
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let neighbour = Searcher::cell(scene, at.offset(dx, dy, dz)).kind;
                if let Some(kind) = neighbour.as_neighbour() {
                    return kind;
                }
            }
        }
    }
    PathType::Walkable
}
