use alloc::vec::Vec;

use glam::DVec3;
use mcrs_voxel::BlockPos;

use crate::PathWorld;

const REGION_MARGIN: f32 = 8.0;
const CHUNK: i32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RequestKind {
    Point,
    Blocks,
    Entity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub kind: RequestKind,
    pub targets: Vec<BlockPos>,
    pub reach: i32,
    pub speed_modifier: f32,
    pub length: f32,
    pub node_multiplier: f32,
}

impl Request {
    pub const DEFAULT_REACH: i32 = 1;

    #[must_use]
    pub fn to_point(point: DVec3, speed_modifier: f32, length: f32) -> Self {
        Self::new(
            RequestKind::Point,
            alloc::vec![BlockPos::from(point)],
            Self::DEFAULT_REACH,
            speed_modifier,
            length,
        )
    }

    #[must_use]
    pub fn to_entity(feet: DVec3, speed_modifier: f32, length: f32) -> Self {
        Self::new(
            RequestKind::Entity,
            alloc::vec![BlockPos::from(feet)],
            Self::DEFAULT_REACH,
            speed_modifier,
            length,
        )
    }

    #[must_use]
    pub fn new(
        kind: RequestKind,
        mut targets: Vec<BlockPos>,
        reach: i32,
        speed_modifier: f32,
        length: f32,
    ) -> Self {
        targets.sort_unstable_by_key(|pos| (pos.x, pos.y, pos.z));
        targets.dedup();
        Self {
            kind,
            targets,
            reach,
            speed_modifier,
            length,
            node_multiplier: 1.0,
        }
    }

    #[must_use]
    pub fn node_budget(&self) -> u32 {
        let budget = libm::floorf(self.length * 16.0) * self.node_multiplier;
        if budget <= 0.0 { 0 } else { budget as u32 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Region {
    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,
}

impl Region {
    #[must_use]
    pub fn around(centre: BlockPos, length: f32) -> Self {
        let reach = (length + REGION_MARGIN) as i32;
        let low_x = (centre.x - reach).div_euclid(CHUNK) * CHUNK;
        let low_z = (centre.z - reach).div_euclid(CHUNK) * CHUNK;
        let high_x = (centre.x + reach).div_euclid(CHUNK) * CHUNK + CHUNK - 1;
        let high_z = (centre.z + reach).div_euclid(CHUNK) * CHUNK + CHUNK - 1;
        Self {
            min_x: low_x,
            min_z: low_z,
            max_x: high_x,
            max_z: high_z,
        }
    }

    #[must_use]
    pub const fn contains(&self, x: i32, z: i32) -> bool {
        self.min_x <= x && x <= self.max_x && self.min_z <= z && z <= self.max_z
    }
}

#[must_use]
pub fn can_update_path(on_ground: bool, in_liquid: bool, passenger: bool) -> bool {
    on_ground || in_liquid || passenger
}

#[must_use]
pub fn adjust_target<W: PathWorld + ?Sized>(world: &W, target: BlockPos) -> BlockPos {
    let min_y = world.min_y();
    let max_y = world.max_y();
    let mut found = None;
    if world.cell(target).is_air() {
        let mut y = target.y - 1;
        while y > min_y {
            let at = BlockPos::new(target.x, y, target.z);
            if !world.cell(at).is_air() {
                return BlockPos::new(target.x, y + 1, target.z);
            }
            y -= 1;
        }
        let mut y = target.y + 1;
        while y <= max_y {
            let at = BlockPos::new(target.x, y, target.z);
            if !world.cell(at).is_air() {
                found = Some(at);
                break;
            }
            y += 1;
        }
    }
    let mut p = found.unwrap_or(target);
    if !world.cell(p).is_solid() {
        return p;
    }
    p = p.offset(0, 1, 0);
    while p.y <= max_y && world.cell(p).is_solid() {
        p = p.offset(0, 1, 0);
    }
    p
}
