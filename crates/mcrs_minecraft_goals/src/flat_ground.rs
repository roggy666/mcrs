use glam::DVec3;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_goal_world::CellFlags;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_minecraft_goal_world::GroundCell;
use mcrs_minecraft_pathfind::CellBits;
use mcrs_minecraft_pathfind::MobPath;
use mcrs_minecraft_pathfind::PathCell;
use mcrs_minecraft_pathfind::PathSearchWorld;
use mcrs_minecraft_pathfind::PathType;
use mcrs_minecraft_pathfind::PathWorld;
use mcrs_voxel::BlockPos;

const WATER_MALUS: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlatGround {
    pub side: i32,
    pub pond: Option<Pond>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pond {
    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,
}

impl Pond {
    #[must_use]
    pub fn holds(&self, at: BlockPos) -> bool {
        at.y == 0
            && self.min_x <= at.x
            && at.x <= self.max_x
            && self.min_z <= at.z
            && at.z <= self.max_z
    }
}

impl GoalWorld for FlatGround {
    fn build_height(&self) -> BuildHeight {
        BuildHeight { min: -64, max: 319 }
    }

    fn cell(&self, at: BlockPos) -> GroundCell {
        let inside = (0..self.side).contains(&at.x) && (0..self.side).contains(&at.z);
        if !inside {
            return GroundCell {
                flags: CellFlags::LEGACY_SOLID,
                malus: -1.0,
                walk_value: 0.0,
            };
        }
        match at.y {
            ..0 => GroundCell {
                flags: CellFlags::LEGACY_SOLID.union(CellFlags::STABLE_FLOOR),
                malus: -1.0,
                walk_value: 0.0,
            },
            0 if self.pond.is_some_and(|pond| pond.holds(at)) => GroundCell {
                flags: CellFlags::STABLE_FLOOR
                    .union(CellFlags::NO_COLLISION)
                    .union(CellFlags::WATER),
                malus: WATER_MALUS,
                walk_value: 0.0,
            },
            0 => GroundCell {
                flags: CellFlags::STABLE_FLOOR
                    .union(CellFlags::NO_COLLISION)
                    .union(CellFlags::GRASS_BELOW),
                malus: 0.0,
                walk_value: 10.0,
            },
            _ => GroundCell {
                flags: CellFlags::NO_COLLISION,
                malus: 0.0,
                walk_value: 0.5,
            },
        }
    }

    fn line_of_sight(&self, _from: DVec3, _to: DVec3) -> bool {
        true
    }
}

impl PathWorld for FlatGround {
    fn path_cell(&self, at: BlockPos) -> PathCell {
        let cell = GoalWorld::cell(self, at);
        if cell.flags.contains(CellFlags::LEGACY_SOLID) {
            PathCell {
                kind: PathType::Blocked,
                top: 1.0,
                bits: CellBits::SHAPE.union(CellBits::SOLID),
            }
        } else if cell.flags.contains(CellFlags::WATER) {
            PathCell {
                kind: PathType::Water,
                top: 0.0,
                bits: CellBits::FLOATABLE,
            }
        } else {
            PathCell::AIR
        }
    }

    fn min_y(&self) -> i32 {
        -64
    }

    fn max_y(&self) -> i32 {
        319
    }

    fn column_loaded(&self, _: BlockPos) -> bool {
        true
    }

    fn collides(&self, min: DVec3, max: DVec3) -> bool {
        let low = BlockPos::from(min);
        let high = BlockPos::from(max - DVec3::splat(1.0e-7));
        (low.x..=high.x).any(|x| {
            (low.y..=high.y).any(|y| {
                (low.z..=high.z).any(|z| {
                    GoalWorld::cell(self, BlockPos::new(x, y, z))
                        .flags
                        .contains(CellFlags::LEGACY_SOLID)
                })
            })
        })
    }
}

impl PathSearchWorld for FlatGround {
    type Search<'a> = &'a FlatGround;

    fn open(&self, _: &MobPath) -> Self::Search<'_> {
        self
    }
}
