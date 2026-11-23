use mcrs_minecraft_path_type::PathType;
use mcrs_voxel::BlockPos;

use crate::NavFlags;
use crate::NavKind;
use crate::SearchWorld;
use crate::Searcher;
use crate::searcher::Scene;
use crate::searcher::neighbour_check;

const SMALL_MOB: f64 = 1.0;
const SMALL_MOB_GROWN: f32 = 1.1;
pub const START_CANDIDATES: usize = 10;

pub(crate) fn block_path_type<W: SearchWorld + ?Sized>(
    scene: &mut Scene<'_, W>,
    at: BlockPos,
) -> PathType {
    let here = Searcher::cell(scene, at).kind;
    let mut kind = here;
    if here == PathType::Open && at.y > scene.world.min_y() {
        let below_pos = at.below();
        let below = Searcher::cell(scene, below_pos).kind;
        kind = match below {
            PathType::Fire | PathType::Lava => PathType::Fire,
            PathType::Damaging => PathType::Damaging,
            PathType::AvoidInAir => PathType::AvoidInAir,
            PathType::Fence => {
                if below_pos == scene.mob_block {
                    PathType::Open
                } else {
                    PathType::Fence
                }
            }
            PathType::Walkable | PathType::Open | PathType::Water => PathType::Open,
            _ => PathType::Walkable,
        };
    }
    if matches!(kind, PathType::Walkable | PathType::Open) {
        kind = neighbour_check(scene, at, kind);
    }
    kind
}

#[must_use]
pub fn start_candidates(
    min: glam::DVec3,
    max: glam::DVec3,
    mut draw: impl FnMut(u32) -> u32,
) -> [BlockPos; START_CANDIDATES] {
    let grow = |extent: f64| f64::from((SMALL_MOB_GROWN - extent as f32).max(0.0));
    let pad = glam::DVec3::new(
        grow(max.x - min.x),
        grow(max.y - min.y),
        grow(max.z - min.z),
    );
    let low = BlockPos::from(min - pad);
    let high = BlockPos::from(max + pad);
    let span = |a: i32, b: i32| u32::try_from(b - a + 1).unwrap_or(1).max(1);
    let (width, height, depth) = (
        span(low.x, high.x),
        span(low.y, high.y),
        span(low.z, high.z),
    );
    core::array::from_fn(|_| {
        let x = draw(width);
        let y = draw(height);
        let z = draw(depth);
        BlockPos::new(
            low.x.saturating_add(x as i32),
            low.y.saturating_add(y as i32),
            low.z.saturating_add(z as i32),
        )
    })
}

pub(crate) fn start_node<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
) -> u32 {
    let mob = scene.mob;
    let block = scene.mob_block;
    let sy = if mob.can(NavFlags::CAN_FLOAT) && mob.in_liquid {
        let mut sy = block.y;
        while Searcher::cell(scene, BlockPos::new(block.x, sy, block.z)).floatable() {
            sy += 1;
        }
        sy
    } else {
        libm::floor(mob.position.y + 0.5) as i32
    };
    let mut start = BlockPos::new(block.x, sy, block.z);
    if !can_start(searcher, scene, start) {
        let size =
            ((mob.max.x - mob.min.x) + (mob.max.y - mob.min.y) + (mob.max.z - mob.min.z)) / 3.0;
        let corners = [
            (mob.min.x, mob.min.z),
            (mob.min.x, mob.max.z),
            (mob.max.x, mob.min.z),
            (mob.max.x, mob.max.z),
        ]
        .map(|(x, z)| BlockPos::new(libm::floor(x) as i32, block.y, libm::floor(z) as i32));
        let candidates: &[BlockPos] = if size >= SMALL_MOB {
            &corners
        } else {
            match &mob.nav {
                NavKind::Flying { starts } => starts,
                NavKind::Ground => &[],
            }
        };
        if let Some(&found) = candidates
            .iter()
            .find(|&&candidate| can_start(searcher, scene, candidate))
        {
            start = found;
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
    searcher.malus(scene, at).1 >= 0.0
}

fn accepted<W: SearchWorld + ?Sized>(
    searcher: &mut Searcher,
    scene: &mut Scene<'_, W>,
    at: BlockPos,
) -> Option<u32> {
    let (kind, malus) = searcher.malus(scene, at);
    if malus < 0.0 {
        return None;
    }
    let index = searcher.node_at(at);
    if let Some(node) = searcher.node_mut(index) {
        node.kind = kind;
        node.malus = node.malus.max(malus);
        if kind == PathType::Walkable {
            node.malus += 1.0;
        }
    }
    Some(index)
}

fn has(searcher: &Searcher, candidate: Option<u32>) -> bool {
    candidate
        .and_then(|index| searcher.node(index))
        .is_some_and(|node| node.malus >= 0.0)
}

fn open(searcher: &Searcher, candidate: Option<u32>) -> bool {
    candidate
        .and_then(|index| searcher.node(index))
        .is_some_and(|node| !node.closed)
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
    let p = node.pos;
    let mut probe = |dx: i32, dy: i32, dz: i32| accepted(searcher, scene, p.offset(dx, dy, dz));
    let s = probe(0, 0, 1);
    let w = probe(-1, 0, 0);
    let e = probe(1, 0, 0);
    let n = probe(0, 0, -1);
    let u = probe(0, 1, 0);
    let d = probe(0, -1, 0);
    let su = probe(0, 1, 1);
    let wu = probe(-1, 1, 0);
    let eu = probe(1, 1, 0);
    let nu = probe(0, 1, -1);
    let sd = probe(0, -1, 1);
    let wd = probe(-1, -1, 0);
    let ed = probe(1, -1, 0);
    let nd = probe(0, -1, -1);
    let ne = probe(1, 0, -1);
    let se = probe(1, 0, 1);
    let nw = probe(-1, 0, -1);
    let sw = probe(-1, 0, 1);
    let neu = probe(1, 1, -1);
    let seu = probe(1, 1, 1);
    let nwu = probe(-1, 1, -1);
    let swu = probe(-1, 1, 1);
    let ned = probe(1, -1, -1);
    let sed = probe(1, -1, 1);
    let nwd = probe(-1, -1, -1);
    let swd = probe(-1, -1, 1);
    let rules: [(Option<u32>, &[Option<u32>]); 26] = [
        (s, &[]),
        (w, &[]),
        (e, &[]),
        (n, &[]),
        (u, &[]),
        (d, &[]),
        (su, &[s, u]),
        (wu, &[w, u]),
        (eu, &[e, u]),
        (nu, &[n, u]),
        (sd, &[s, d]),
        (wd, &[w, d]),
        (ed, &[e, d]),
        (nd, &[n, d]),
        (ne, &[n, e]),
        (se, &[s, e]),
        (nw, &[n, w]),
        (sw, &[s, w]),
        (neu, &[ne, n, e, u, nu, eu]),
        (seu, &[se, s, e, u, su, eu]),
        (nwu, &[nw, n, w, u, nu, wu]),
        (swu, &[sw, s, w, u, su, wu]),
        (ned, &[ne, n, e, d, nd, ed]),
        (sed, &[se, s, e, d, sd, ed]),
        (nwd, &[nw, n, w, d, nd, wd]),
        (swd, &[sw, s, w, d, sd, wd]),
    ];
    for (candidate, needed) in rules {
        if open(searcher, candidate)
            && needed.iter().all(|&need| has(searcher, need))
            && let Some(index) = candidate
        {
            out.push(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use glam::DVec3;
    use mcrs_minecraft_path_type::PathType;
    use mcrs_voxel::BlockPos;

    use crate::MobPath;
    use crate::NavFlags;
    use crate::NavKind;
    use crate::Request;
    use crate::RequestKind;
    use crate::Searcher;
    use crate::fixture::MapWorld;
    use crate::fixture::STONE;
    use crate::fixture::WATER;
    use crate::search;

    const FLOOR: i32 = 10;
    const PARROT_RANGE: f32 = 16.0;

    fn parrot_at(x: f64, y: f64, z: f64) -> MobPath {
        let mut mob = MobPath::standing(DVec3::new(x, y, z), 0.5, 0.9, 0.6);
        mob.on_ground = false;
        mob.flags = NavFlags::DEFAULT.union(NavFlags::CAN_FLOAT);
        mob.malus.set(PathType::FireInNeighbor, -1.0);
        mob.malus.set(PathType::Fire, -1.0);
        mob.malus.set(PathType::AvoidInAir, -1.0);
        let mut draws = 0u32;
        mob.nav = NavKind::Flying {
            starts: super::start_candidates(mob.min, mob.max, |n| {
                draws = draws.wrapping_add(7);
                draws % n
            }),
        };
        mob
    }

    fn fly(world: &MapWorld, mob: &MobPath, to: BlockPos) -> Option<Vec<BlockPos>> {
        let request = Request::new(
            RequestKind::Blocks,
            alloc::vec![to],
            Request::DEFAULT_REACH,
            1.0,
            PARROT_RANGE,
        );
        let mut searcher = Searcher::default();
        let mut world = world;
        let outcome = search(&mut world, mob, &request, &mut searcher);
        outcome
            .path
            .filter(|path| path.reached)
            .map(|path| path.nodes.iter().map(|node| node.pos()).collect())
    }

    #[test]
    fn fly_003_the_start_node_is_the_block_of_the_rounded_height() {
        let world = MapWorld::flat(FLOOR);
        let mob = parrot_at(0.5, f64::from(FLOOR) + 5.6, 0.5);
        let path = fly(&world, &mob, BlockPos::new(0, FLOOR + 16, 0)).expect("a path up");
        assert_eq!(path.first(), Some(&BlockPos::new(0, FLOOR + 6, 0)));
    }

    #[test]
    fn fly_004_a_flight_through_the_air_climbs_over_a_wall_with_diagonal_steps() {
        let mut world = MapWorld::flat(FLOOR);
        world.wall(3, -2, 2, FLOOR + 1, FLOOR + 1);
        let mob = parrot_at(0.5, f64::from(FLOOR) + 1.0, 0.5);
        let path = fly(&world, &mob, BlockPos::new(6, FLOOR + 1, 0)).expect("a path over the wall");
        assert!(
            path.iter()
                .all(|node| node.x != 3 || node.y > FLOOR + 1 || node.z.abs() > 2),
            "the path does not enter the wall: {path:?}"
        );
        let climbs = path.iter().any(|node| node.y > FLOOR + 1);
        assert!(climbs, "the path climbs over the wall: {path:?}");
        let diagonal = path
            .windows(2)
            .any(|pair| pair[0].x != pair[1].x && pair[0].y != pair[1].y);
        assert!(diagonal, "the path uses a diagonal step: {path:?}");
    }

    #[test]
    fn fly_004_the_walked_distance_of_a_touched_node_is_overwritten_as_in_the_reference() {
        let mut world = MapWorld::flat(FLOOR);
        world.wall(4, -3, 3, FLOOR + 1, FLOOR + 3);
        let mob = parrot_at(0.5, f64::from(FLOOR) + 1.0, 0.5);
        let request = Request::new(
            RequestKind::Blocks,
            alloc::vec![BlockPos::new(7, FLOOR + 1, 0)],
            Request::DEFAULT_REACH,
            1.0,
            PARROT_RANGE,
        );
        let mut searcher = Searcher::default();
        let mut view = &world;
        let outcome = search(&mut view, &mob, &request, &mut searcher);
        assert!(
            outcome.path.is_some_and(|path| !path.reached),
            "a parrot with the range 16 gives a partial path at this wall"
        );
        assert!(
            outcome.pops < 256,
            "the open set runs dry before the budget: {}",
            outcome.pops
        );
    }

    #[test]
    fn fly_002_the_air_over_water_stays_open_and_the_air_beside_it_is_a_border() {
        let mut world = MapWorld::flat(FLOOR);
        for x in 2..=4 {
            for z in -1..=1 {
                world.set(BlockPos::new(x, FLOOR, z), WATER);
            }
        }
        let mut bee = parrot_at(0.5, f64::from(FLOOR) + 1.0, 0.5);
        bee.flags = NavFlags::DEFAULT;
        bee.malus.set(PathType::Water, -1.0);
        bee.malus.set(PathType::WaterBorder, 16.0);
        let path =
            fly(&world, &bee, BlockPos::new(6, FLOOR + 1, 0)).expect("a path past the water");
        let over_water_low = path
            .iter()
            .any(|node| (2..=4).contains(&node.x) && node.y <= FLOOR + 1 && node.z.abs() <= 1);
        assert!(!over_water_low, "the bee avoids the water border: {path:?}");
    }

    #[test]
    fn fly_004_a_walkable_node_pays_one_more_for_each_expansion_that_reaches_it() {
        let mut world = MapWorld::flat(FLOOR);
        world.set(BlockPos::new(2, FLOOR + 1, 0), STONE);
        let mob = parrot_at(0.5, f64::from(FLOOR) + 1.0, 0.5);
        let request = Request::new(
            RequestKind::Blocks,
            alloc::vec![BlockPos::new(5, FLOOR + 1, 0)],
            Request::DEFAULT_REACH,
            1.0,
            PARROT_RANGE,
        );
        let mut searcher = Searcher::default();
        let mut view = &world;
        let outcome = search(&mut view, &mob, &request, &mut searcher);
        assert!(outcome.path.is_some_and(|path| path.reached));
        let over_stone = searcher
            .nodes
            .iter()
            .find(|node| node.pos == BlockPos::new(2, FLOOR + 2, 0))
            .expect("the node over the stone was probed");
        assert_eq!(over_stone.kind, PathType::Walkable);
        assert!(over_stone.malus >= 2.0, "malus {}", over_stone.malus);
    }
}
