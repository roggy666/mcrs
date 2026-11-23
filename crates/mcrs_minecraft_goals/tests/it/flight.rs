use core::f32::consts::FRAC_PI_2;
use glam::DVec3;
use mcrs_minecraft_brain::KeyedRandom;
use mcrs_minecraft_brain::KindData;
use mcrs_minecraft_brain::KindFlags;
use mcrs_minecraft_brain::Landing;
use mcrs_minecraft_brain::Part;
use mcrs_minecraft_brain::WorldSeed;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_goal_world::CellFlags;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_minecraft_goal_world::GroundCell;
use mcrs_minecraft_pathfind::PathType;
use mcrs_voxel::BlockPos;

use mcrs_minecraft_goals::*;

const PARROT: KindData = KindData::new(
    mcrs_minecraft_brain::Dimensions::new(0.5, 0.9, 0.54),
    KindFlags::CAN_BE_TAME,
)
.with_malus(&[(PathType::AvoidInAir, -1.0)]);

fn random(tick: u64) -> KeyedRandom {
    KeyedRandom::new(WorldSeed(7), &EntityUuid::from_u128(3), tick, Part(0x0E00))
}

#[test]
fn pos_007_a_directional_offset_stays_in_its_box_and_leans_forward() {
    let heading = Heading::of_yaw(0.0);
    let mut forward = 0;
    let mut draws = 0;
    for tick in 0..200 {
        let mut random = random(tick);
        if let Some(offset) = directional_offset(&mut random, 0.0, 8.0, 7, -2, heading, FRAC_PI_2) {
            draws += 1;
            assert!(offset.x.abs() <= 8 && offset.z.abs() <= 8, "{offset:?}");
            assert!((-9..=5).contains(&offset.y), "{offset:?}");
            forward += i32::from(offset.z > 0);
        }
    }
    assert!(draws > 100, "most draws give an offset: {draws}");
    assert!(
        forward * 10 > draws * 8,
        "yaw 0 looks along +z: {forward} of {draws}"
    );
}

#[test]
fn pos_008_the_hover_search_lifts_a_buried_candidate_one_to_three_over_the_floor() {
    let world = FlatGround {
        side: 64,
        pond: None,
    };
    let feet = DVec3::new(32.5, 0.0, 32.5);
    let mut found = 0;
    for tick in 0..60 {
        let mut random = random(tick);
        if let Some(point) = hover_search(
            &world,
            &PARROT,
            Landing::StandOn,
            feet,
            Heading::of_yaw(90.0),
            &mut random,
        ) {
            found += 1;
            assert!((0.0..=3.0).contains(&point.y), "{point:?}");
            assert_eq!(point.x.fract(), 0.5);
        }
    }
    assert!(
        found > 40,
        "the hover search finds a point in most ticks: {found}"
    );
}

#[test]
fn pos_009_the_air_search_accepts_the_air_and_rises_out_of_the_ground() {
    let world = FlatGround {
        side: 64,
        pond: None,
    };
    let feet = DVec3::new(32.5, 3.0, 32.5);
    let mut random = random(11);
    let point = air_search(&world, &PARROT, feet, Heading::of_yaw(180.0), &mut random)
        .expect("an air point");
    assert!((0.0..=5.0).contains(&point.y), "{point:?}");
}

struct Tree;

impl GoalWorld for Tree {
    fn build_height(&self) -> BuildHeight {
        BuildHeight { min: -64, max: 319 }
    }

    fn cell(&self, at: BlockPos) -> GroundCell {
        let leaves = at.x == 4 && at.z == 0 && (3..=5).contains(&at.y);
        let air = at.y > 0 && !leaves;
        let mut flags = CellFlags::NONE;
        if air {
            flags = flags.union(CellFlags::AIR);
        }
        let below = BlockPos::new(at.x, at.y - 1, at.z);
        if below.x == 4 && below.z == 0 && (3..=5).contains(&below.y) {
            flags = flags.union(CellFlags::LEAVES_BELOW);
        }
        GroundCell {
            flags,
            malus: 0.0,
            walk_value: 0.5,
            path: if air {
                PathType::Open
            } else {
                PathType::Blocked
            },
        }
    }

    fn line_of_sight(&self, _: DVec3, _: DVec3) -> bool {
        true
    }
}

#[test]
fn fly_001_the_tree_position_is_the_first_air_block_over_leaves_in_x_y_z_order() {
    let perch = tree_position(&Tree, DVec3::new(2.5, 1.0, 0.5)).expect("a perch");
    assert_eq!(perch, DVec3::new(4.5, 6.0, 0.5));
    assert!(tree_position(&Tree, DVec3::new(20.5, 1.0, 0.5)).is_none());
}
