use core::f32::consts::FRAC_PI_2;
use core::f32::consts::SQRT_2;

use glam::DVec3;
use mcrs_minecraft_brain::KeyedRandom;
use mcrs_minecraft_brain::KindData;
use mcrs_minecraft_brain::Landing;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::Turn;
use mcrs_minecraft_goal_world::CellFlags;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_minecraft_goal_world::GroundCell;
use mcrs_minecraft_numeric as numeric;
use mcrs_voxel::BlockPos;

use crate::BrainWorld;
use crate::Gate;
use crate::WANDER_WAIT;
use crate::random_position;

const TRIES: u32 = 10;
pub const HOVER_HORIZONTAL: u32 = 8;
pub const HOVER_VERTICAL: u32 = 7;
pub const HOVER_LOW: u32 = 1;
pub const HOVER_HIGH: u32 = 3;
pub const AIR_HORIZONTAL: u32 = 8;
pub const AIR_VERTICAL: u32 = 4;
pub const AIR_HEIGHT: i32 = -2;
pub const TREE_CHANCE: f32 = 0.001;
pub const TREE_REACH: i32 = 3;
pub const TREE_HEIGHT: i32 = 6;
pub const WANDER_CHANCE: f64 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Heading {
    pub dx: f64,
    pub dz: f64,
}

impl Heading {
    #[must_use]
    pub fn of_yaw(yaw: f32) -> Self {
        let angle = -numeric::radians_from_degrees(yaw);
        Self {
            dx: f64::from(numeric::sin_nearest(angle)),
            dz: f64::from(numeric::cos_nearest(angle)),
        }
    }
}

#[must_use]
pub fn directional_offset(
    random: &mut KeyedRandom,
    low: f64,
    high: f64,
    vertical: u32,
    height: i32,
    heading: Heading,
    angle: f32,
) -> Option<BlockPos> {
    let centre = libm::atan2(heading.dz, heading.dx) - f64::from(FRAC_PI_2);
    let spread = 2.0_f32 * random.unit_f32() - 1.0_f32;
    let theta = centre + f64::from(spread) * f64::from(angle);
    let unit = libm::sqrt(random.unit());
    let length = (low + (high - low) * unit) * f64::from(SQRT_2);
    let ox = -length * libm::sin(theta);
    let oz = length * libm::cos(theta);
    if ox.abs() > high || oz.abs() > high {
        return None;
    }
    let oy = random.symmetric(vertical) + height;
    Some(BlockPos::from(DVec3::new(ox, f64::from(oy), oz)))
}

fn best_of(
    random: &mut KeyedRandom,
    mut judge: impl FnMut(&mut KeyedRandom) -> Option<(BlockPos, f32)>,
) -> Option<DVec3> {
    let mut best: Option<(BlockPos, f32)> = None;
    for _ in 0..TRIES {
        let Some((position, value)) = judge(random) else {
            continue;
        };
        if best.is_none_or(|(_, best_value)| value > best_value) {
            best = Some((position, value));
        }
    }
    best.map(|(position, _)| position.bottom_centre())
}

fn toward(feet: DVec3, offset: BlockPos) -> BlockPos {
    BlockPos::from(DVec3::new(
        f64::from(offset.x) + feet.x,
        f64::from(offset.y) + feet.y,
        f64::from(offset.z) + feet.z,
    ))
}

fn lands<W: GoalWorld + ?Sized>(world: &W, landing: Landing, at: BlockPos) -> bool {
    let cell = world.cell(at);
    match landing {
        Landing::StandOn => cell.flags.contains(CellFlags::STANDABLE),
        Landing::NotAir => !cell.flags.contains(CellFlags::AIR_BELOW),
    }
}

fn solid<W: GoalWorld + ?Sized>(world: &W, at: BlockPos) -> bool {
    world.cell(at).flags.contains(CellFlags::LEGACY_SOLID)
}

fn passable(kind: &KindData, cell: &GroundCell) -> bool {
    kind.malus_of(cell.path) == 0.0
}

#[must_use]
pub fn hover_search<W: GoalWorld + ?Sized>(
    world: &W,
    kind: &KindData,
    landing: Landing,
    feet: DVec3,
    heading: Heading,
    random: &mut KeyedRandom,
) -> Option<DVec3> {
    let height = world.build_height();
    best_of(random, |random| {
        let offset = directional_offset(
            random,
            0.0,
            f64::from(HOVER_HORIZONTAL),
            HOVER_VERTICAL,
            0,
            heading,
            FRAC_PI_2,
        )?;
        let mut at = toward(feet, offset);
        if !height.contains(at.y) || !lands(world, landing, at) {
            return None;
        }
        if solid(world, at) {
            while at.y <= height.max && solid(world, at) {
                at = at.offset(0, 1, 0);
            }
            let base = at.y;
            let lift = random.below(HOVER_HIGH - HOVER_LOW + 1) + HOVER_LOW;
            while at.y <= height.max && (at.y - base) < lift as i32 {
                at = at.offset(0, 1, 0);
                if solid(world, at) {
                    at = at.below();
                    break;
                }
            }
        }
        let cell = world.cell(at);
        let valid = !cell.flags.contains(CellFlags::WATER) && passable(kind, &cell);
        valid.then_some((at, cell.walk_value))
    })
}

#[must_use]
pub fn air_search<W: GoalWorld + ?Sized>(
    world: &W,
    kind: &KindData,
    feet: DVec3,
    heading: Heading,
    random: &mut KeyedRandom,
) -> Option<DVec3> {
    let height = world.build_height();
    best_of(random, |random| {
        let offset = directional_offset(
            random,
            0.0,
            f64::from(AIR_HORIZONTAL),
            AIR_VERTICAL,
            AIR_HEIGHT,
            heading,
            FRAC_PI_2,
        )?;
        let mut at = toward(feet, offset);
        if !height.contains(at.y) {
            return None;
        }
        while at.y <= height.max && solid(world, at) {
            at = at.offset(0, 1, 0);
        }
        let cell = world.cell(at);
        passable(kind, &cell).then_some((at, cell.walk_value))
    })
}

#[must_use]
pub fn tree_position<W: GoalWorld + ?Sized>(world: &W, feet: DVec3) -> Option<DVec3> {
    let block = BlockPos::from(feet);
    let low = BlockPos::from(DVec3::new(
        feet.x - f64::from(TREE_REACH),
        feet.y - f64::from(TREE_HEIGHT),
        feet.z - f64::from(TREE_REACH),
    ));
    let high = BlockPos::from(DVec3::new(
        feet.x + f64::from(TREE_REACH),
        feet.y + f64::from(TREE_HEIGHT),
        feet.z + f64::from(TREE_REACH),
    ));
    for z in low.z..=high.z {
        for y in low.y..=high.y {
            for x in low.x..=high.x {
                let at = BlockPos::new(x, y, z);
                if at == block {
                    continue;
                }
                let cell = world.cell(at);
                let perch = cell.flags.contains(CellFlags::LEAVES_BELOW)
                    || cell.flags.contains(CellFlags::LOG_BELOW);
                if perch
                    && cell.flags.contains(CellFlags::AIR)
                    && world
                        .cell(at.offset(0, 1, 0))
                        .flags
                        .contains(CellFlags::AIR)
                {
                    return Some(at.bottom_centre());
                }
            }
        }
    }
    None
}

#[must_use]
pub fn stroll_destination<W: BrainWorld + ?Sized>(
    turn: &Turn<'_, W>,
    kind: &KindData,
    landing: Landing,
    random: &mut KeyedRandom,
) -> Option<DVec3> {
    let world = turn.world;
    let view = turn.view;
    let feet = view.position;
    let mut found = None;
    if view.flags.contains(MobFlags::IN_WATER) {
        found = random_position::land_search(world, feet, crate::SearchRange::new(15, 15), random);
    }
    if random.unit_f32() >= TREE_CHANCE {
        found = tree_position(world, feet);
    }
    if found.is_some() {
        return found;
    }
    let heading = Heading::of_yaw(view.yaw);
    hover_search(world, kind, landing, feet, heading, random)
        .or_else(|| air_search(world, kind, feet, heading, random))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WanderGoal {
    gate: Gate,
}

impl WanderGoal {
    pub fn start_destination<W: GoalWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        kind: &KindData,
        landing: Landing,
        path_running: bool,
    ) -> Option<DVec3> {
        if path_running {
            self.gate.end();
            return None;
        }
        if !self.gate.passes(turn, WANDER_WAIT, WANDER_CHANCE) {
            return None;
        }
        let mut random = turn.random(crate::WANDER_SEARCH);
        let feet = turn.view.position;
        let heading = Heading::of_yaw(turn.view.yaw);
        hover_search(turn.world, kind, landing, feet, heading, &mut random)
            .or_else(|| air_search(turn.world, kind, feet, heading, &mut random))
    }

    #[must_use]
    pub const fn gate(&self) -> Gate {
        self.gate
    }

    pub fn gate_mut(&mut self) -> &mut Gate {
        &mut self.gate
    }
}
