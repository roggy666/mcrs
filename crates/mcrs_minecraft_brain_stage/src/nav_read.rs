use mcrs_minecraft_goal_world::CellFlags;
use mcrs_minecraft_goal_world::GroundCell;
use mcrs_minecraft_nav_projection::NavRow;
use mcrs_minecraft_pathfind::CellBits;
use mcrs_minecraft_pathfind::PathCell;
use mcrs_minecraft_physics::BlockFlags;
use mcrs_minecraft_physics::CollisionTable;
use mcrs_minecraft_physics::PathType;
use mcrs_minecraft_physics::ShapeClass;
use mcrs_voxel::BlockPos;

const GRASS_WALK_VALUE: f32 = 10.0;
const OTHER_WALK_VALUE: f32 = 0.5;

pub trait NavRows {
    fn row(&self, at: BlockPos) -> NavRow;

    fn lowest_y(&self) -> i32;
}

#[must_use]
pub fn block_path_type<R: NavRows + ?Sized>(
    rows: &R,
    at: BlockPos,
    here: NavRow,
    below: NavRow,
) -> PathType {
    if here.path != PathType::Open || at.y <= rows.lowest_y() {
        return here.path;
    }
    if let Some(path) = PathType::over(below.path) {
        return path;
    }
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let neighbour = rows.row(at.offset(dx, dy, dz)).path;
                if let Some(path) = neighbour.as_neighbour() {
                    return path;
                }
            }
        }
    }
    PathType::Walkable
}

#[must_use]
pub fn ground_cell<R: NavRows + ?Sized>(
    rows: &R,
    shapes: &CollisionTable,
    at: BlockPos,
) -> GroundCell {
    let here = rows.row(at);
    let below = rows.row(at.below());
    let marks = [
        (
            below.flags.contains(BlockFlags::SOLID_RENDER),
            CellFlags::STABLE_FLOOR,
        ),
        (
            here.flags.contains(BlockFlags::LEGACY_SOLID),
            CellFlags::LEGACY_SOLID,
        ),
        (here.flags.contains(BlockFlags::WATER), CellFlags::WATER),
        (
            shapes.shape_without_context(here.class).is_none(),
            CellFlags::NO_COLLISION,
        ),
    ];
    let flags = marks
        .into_iter()
        .filter(|(set, _)| *set)
        .fold(CellFlags::NONE, |flags, (_, flag)| flags.union(flag));
    GroundCell {
        flags,
        malus: block_path_type(rows, at, here, below).default_malus(),
        walk_value: if below.flags.contains(BlockFlags::GRASS_BLOCK) {
            GRASS_WALK_VALUE
        } else {
            OTHER_WALK_VALUE
        },
    }
}

#[must_use]
pub fn feet_top(shapes: &CollisionTable, row: NavRow, y: i32) -> Option<f64> {
    if row.flags.intersects(BlockFlags::DOOR | BlockFlags::FENCE) {
        return None;
    }
    let shape = shapes.shape_without_context(row.class)?;
    Some(f64::from(y) + shape.top())
}

#[must_use]
pub fn path_cell_of(shapes: &CollisionTable, row: NavRow) -> PathCell {
    let top = shapes
        .shape_without_context(row.class)
        .map_or(0.0, |shape| shape.top() as f32);
    let bits = CellBits::NONE
        .with(CellBits::AIR, row.flags.contains(BlockFlags::AIR))
        .with(CellBits::SHAPE, row.class != ShapeClass::Empty)
        .with(
            CellBits::SOLID,
            row.flags.contains(BlockFlags::LEGACY_SOLID),
        )
        .with(CellBits::FLOATABLE, row.flags.contains(BlockFlags::FLUID))
        .with(CellBits::CAULDRON, row.flags.contains(BlockFlags::CAULDRON))
        .with(
            CellBits::DOOR_OR_FENCE,
            row.flags.intersects(
                BlockFlags::DOOR | BlockFlags::FENCE | BlockFlags::WALL | BlockFlags::FENCE_GATE,
            ),
        );
    PathCell {
        kind: row.path,
        top,
        bits,
    }
}
