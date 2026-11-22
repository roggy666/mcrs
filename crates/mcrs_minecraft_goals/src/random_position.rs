use glam::DVec3;
use mcrs_minecraft_brain::KeyedRandom;
use mcrs_minecraft_goal_world::CellFlags;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_voxel::BlockPos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SearchRange {
    pub horizontal: u32,
    pub vertical: u32,
}

impl SearchRange {
    #[must_use]
    pub const fn new(horizontal: u32, vertical: u32) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

const TRIES: u32 = 10;

pub fn default_search<W: GoalWorld + ?Sized>(
    world: &W,
    feet: DVec3,
    range: SearchRange,
    random: &mut KeyedRandom,
) -> Option<DVec3> {
    best_of_tries(feet, range, random, |candidate| {
        let cell = world.cell(candidate);
        let valid = world.build_height().contains(candidate.y)
            && cell.flags.contains(CellFlags::STABLE_FLOOR)
            && cell.malus == 0.0;
        valid.then_some((candidate, cell.walk_value))
    })
}

pub fn land_search<W: GoalWorld + ?Sized>(
    world: &W,
    feet: DVec3,
    range: SearchRange,
    random: &mut KeyedRandom,
) -> Option<DVec3> {
    let height = world.build_height();
    best_of_tries(feet, range, random, |mut candidate| {
        if !height.contains(candidate.y) {
            return None;
        }
        let mut cell = world.cell(candidate);
        if !cell.flags.contains(CellFlags::STABLE_FLOOR) {
            return None;
        }
        while cell.flags.contains(CellFlags::LEGACY_SOLID) && candidate.y <= height.max {
            candidate = candidate.offset(0, 1, 0);
            cell = world.cell(candidate);
        }
        let valid = !cell.flags.contains(CellFlags::WATER) && cell.malus == 0.0;
        valid.then_some((candidate, cell.walk_value))
    })
}

fn best_of_tries(
    feet: DVec3,
    range: SearchRange,
    random: &mut KeyedRandom,
    mut judge: impl FnMut(BlockPos) -> Option<(BlockPos, f32)>,
) -> Option<DVec3> {
    let mut best: Option<(BlockPos, f32)> = None;
    for _ in 0..TRIES {
        let a = random.symmetric(range.horizontal);
        let b = random.symmetric(range.vertical);
        let c = random.symmetric(range.horizontal);
        let candidate = BlockPos::from(DVec3::new(
            feet.x + f64::from(a),
            feet.y + f64::from(b),
            feet.z + f64::from(c),
        ));
        let Some((position, value)) = judge(candidate) else {
            continue;
        };
        if best.is_none_or(|(_, best_value)| value > best_value) {
            best = Some((position, value));
        }
    }
    best.map(|(position, _)| position.bottom_centre())
}

pub fn away_search<W: GoalWorld + ?Sized>(
    world: &W,
    feet: DVec3,
    away_from: DVec3,
    range: SearchRange,
    random: &mut KeyedRandom,
) -> Option<DVec3> {
    let ax = feet.x - away_from.x;
    let az = feet.z - away_from.z;
    let centre = libm::atan2(az, ax) - core::f64::consts::FRAC_PI_2;
    let horizontal = f64::from(range.horizontal);
    let mut best: Option<(BlockPos, f32)> = None;
    for _ in 0..TRIES {
        let angle =
            centre + (2.0 * f64::from(random.unit_f32()) - 1.0) * core::f64::consts::FRAC_PI_2;
        let length = horizontal * libm::sqrt(random.unit()) * core::f64::consts::SQRT_2;
        let dx = -length * libm::sin(angle);
        let dz = length * libm::cos(angle);
        if dx.abs() > horizontal || dz.abs() > horizontal {
            continue;
        }
        let b = random.symmetric(range.vertical);
        let candidate = BlockPos::from(DVec3::new(feet.x + dx, feet.y + f64::from(b), feet.z + dz));
        let cell = world.cell(candidate);
        let valid = world.build_height().contains(candidate.y)
            && cell.flags.contains(CellFlags::STABLE_FLOOR)
            && cell.malus == 0.0;
        if !valid {
            continue;
        }
        if best.is_none_or(|(_, best_value)| cell.walk_value > best_value) {
            best = Some((candidate, cell.walk_value));
        }
    }
    best.map(|(position, _)| position.bottom_centre())
}
