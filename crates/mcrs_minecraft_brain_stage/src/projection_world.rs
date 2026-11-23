use glam::DVec3;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_minecraft_goal_world::GroundCell;
use mcrs_minecraft_nav_projection::NavRow;
use mcrs_minecraft_nav_projection::NavView;
use mcrs_minecraft_pathfind::MobPath;
use mcrs_minecraft_pathfind::PathCell;
use mcrs_minecraft_pathfind::PathSearchWorld;
use mcrs_minecraft_pathfind::PathWorld;
use mcrs_minecraft_pathfind::SearchWorld;
use mcrs_minecraft_physics::Aabb;
use mcrs_minecraft_physics::BlockFlags;
use mcrs_minecraft_physics::EntityCollider;
use mcrs_minecraft_physics::NoBodies;
use mcrs_minecraft_physics::QueryEntity;
use mcrs_minecraft_physics::View;
use mcrs_minecraft_physics::intersects;
use mcrs_minecraft_physics::sight_clear;
use mcrs_voxel::BlockPos;

use crate::NavRows;
use crate::NavSight;
use crate::StageTables;
use crate::nav_read;

const SIGHT_RANGE: f64 = 128.0;

#[derive(Clone, Copy, Debug)]
pub struct ProjectionWorld<'a> {
    view: NavView<'a>,
    tables: &'a StageTables,
}

impl<'a> ProjectionWorld<'a> {
    #[must_use]
    pub const fn new(view: NavView<'a>, tables: &'a StageTables) -> Self {
        Self { view, tables }
    }

    #[must_use]
    pub const fn view(&self) -> NavView<'a> {
        self.view
    }

    #[must_use]
    pub const fn tables(&self) -> &'a StageTables {
        self.tables
    }

    #[must_use]
    pub fn feet_top(&self, at: BlockPos) -> Option<f64> {
        nav_read::feet_top(&self.tables.physics.collision, self.view.row(at), at.y)
    }

    #[must_use]
    pub fn is_cauldron(&self, at: BlockPos) -> bool {
        self.view.row(at).flags.contains(BlockFlags::CAULDRON)
    }
}

impl NavRows for ProjectionWorld<'_> {
    fn row(&self, at: BlockPos) -> NavRow {
        self.view.row(at)
    }

    fn lowest_y(&self) -> i32 {
        self.view.min_y()
    }
}

impl GoalWorld for ProjectionWorld<'_> {
    fn build_height(&self) -> BuildHeight {
        BuildHeight {
            min: self.view.min_y(),
            max: self.view.max_y(),
        }
    }

    fn cell(&self, at: BlockPos) -> GroundCell {
        nav_read::ground_cell(self, &self.tables.physics.collision, at)
    }

    fn line_of_sight(&self, from: DVec3, to: DVec3) -> bool {
        if (to - from).length_squared() > SIGHT_RANGE * SIGHT_RANGE {
            return false;
        }
        sight_clear(
            &NavSight::new(self.view),
            &self.tables.physics.collision,
            from,
            to,
        )
    }
}

impl PathWorld for ProjectionWorld<'_> {
    fn path_cell(&self, at: BlockPos) -> PathCell {
        nav_read::path_cell_of(&self.tables.physics.collision, self.view.row(at))
    }

    fn min_y(&self) -> i32 {
        self.view.min_y()
    }

    fn max_y(&self) -> i32 {
        self.view.max_y()
    }

    fn column_loaded(&self, at: BlockPos) -> bool {
        self.view.is_loaded(at)
    }

    fn clear_between(&self, from: DVec3, to: DVec3) -> bool {
        sight_clear(
            &NavSight::new(self.view),
            &self.tables.physics.collision,
            from,
            to,
        ) && mcrs_minecraft_pathfind::fluid_free(self, from, to)
    }

    fn collides(&self, min: DVec3, max: DVec3) -> bool {
        let mob = MobPath::standing(
            DVec3::new((min.x + max.x) / 2.0, min.y, (min.z + max.z) / 2.0),
            (max.x - min.x) as f32,
            (max.y - min.y) as f32,
            0.0,
        );
        SearchWorld::collides(&mut ProjectionSearch::new(*self, &mob), min, max)
    }
}

impl<'a> PathSearchWorld for ProjectionWorld<'a> {
    type Search<'s>
        = ProjectionSearch<'s>
    where
        Self: 's;

    fn open(&self, mob: &MobPath) -> Self::Search<'_> {
        ProjectionSearch::new(*self, mob)
    }

    fn version(&self) -> u64 {
        self.view.tick()
    }

    fn reopen(&self, mob: &MobPath, version: u64) -> Option<Self::Search<'_>> {
        let view = self.view.at(version);
        if !view.is_live() || version > self.view.tick() {
            return None;
        }
        Some(ProjectionSearch::new(Self::new(view, self.tables), mob))
    }
}

pub struct ProjectionSearch<'a> {
    world: ProjectionWorld<'a>,
    entity: QueryEntity,
    colliders: Vec<EntityCollider>,
}

impl<'a> ProjectionSearch<'a> {
    #[must_use]
    pub fn new(world: ProjectionWorld<'a>, mob: &MobPath) -> Self {
        Self {
            world,
            entity: QueryEntity {
                key: mob_key(mob),
                position: mob.position,
                fall_distance: 0.0,
                shift: false,
                powder_snow_solid: false,
                walks_on_powder_snow: false,
            },
            colliders: Vec::new(),
        }
    }
}

fn mob_key(mob: &MobPath) -> mcrs_minecraft_entity_key::EntityUuid {
    let bits = u128::from(mob.position.x.to_bits()) << 64 | u128::from(mob.position.z.to_bits());
    mcrs_minecraft_entity_key::EntityUuid::from_u128(bits)
}

impl SearchWorld for ProjectionSearch<'_> {
    fn path_cell(&mut self, at: BlockPos) -> PathCell {
        PathWorld::path_cell(&self.world, at)
    }

    fn min_y(&mut self) -> i32 {
        PathWorld::min_y(&self.world)
    }

    fn max_y(&mut self) -> i32 {
        PathWorld::max_y(&self.world)
    }

    fn column_loaded(&mut self, at: BlockPos) -> bool {
        PathWorld::column_loaded(&self.world, at)
    }

    fn collides(&mut self, min: DVec3, max: DVec3) -> bool {
        let sight = NavSight::new(self.world.view);
        let view = View::new(&sight, &NoBodies, &self.world.tables.physics);
        intersects(view, &self.entity, Aabb::new(min, max), &mut self.colliders)
    }
}
