use glam::DVec3;
use mcrs_minecraft_brain_stage::path_cell_of;
use mcrs_minecraft_collision_table::BlockSection;
use mcrs_minecraft_collision_table::BlockWorld;
use mcrs_minecraft_nav_projection::NavRow;
use mcrs_minecraft_pathfind::MobPath;
use mcrs_minecraft_pathfind::PathCell;
use mcrs_minecraft_pathfind::PathWorld;
use mcrs_minecraft_pathfind::SearchWorld;
use mcrs_minecraft_physics::Aabb;
use mcrs_minecraft_physics::CollisionSection;
use mcrs_minecraft_physics::CollisionWorld;
use mcrs_minecraft_physics::EntityCollider;
use mcrs_minecraft_physics::NoBodies;
use mcrs_minecraft_physics::QueryEntity;
use mcrs_minecraft_physics::SectionLookup;
use mcrs_minecraft_physics::Tables;
use mcrs_minecraft_physics::View;
use mcrs_minecraft_physics::WorldBorder;
use mcrs_minecraft_physics::intersects;
use mcrs_minecraft_volume::BlockVolume;
use mcrs_voxel::BlockPos;
use mcrs_voxel::BlockStateId;
use mcrs_voxel::LocalPos;
use mcrs_voxel::SectionPos;

use crate::BlockGoalWorld;

pub struct SearchView<'a, V: BlockVolume> {
    base: &'a BlockGoalWorld<'a, V>,
    blocks: BlockWorld<'a, V>,
    tables: &'a Tables,
    entity: QueryEntity,
    colliders: Vec<EntityCollider>,
}

impl<'a, V: BlockVolume> SearchView<'a, V> {
    pub(crate) fn new(base: &'a BlockGoalWorld<'a, V>, mob: &MobPath) -> Self {
        Self {
            base,
            blocks: BlockWorld {
                blocks: base.blocks(),
                table: &base.tables().collision,
            },
            tables: base.tables(),
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

    fn state_at(&self, at: BlockPos) -> Option<BlockStateId> {
        match self.blocks.section_at(SectionPos::from(at)) {
            SectionLookup::Loaded(section) => Some(section.state(LocalPos::from(at))),
            SectionLookup::Outside | SectionLookup::Unloaded => None,
        }
    }
}

fn mob_key(mob: &MobPath) -> mcrs_minecraft_entity_key::EntityUuid {
    let bits = u128::from(mob.position.x.to_bits()) << 64 | u128::from(mob.position.z.to_bits());
    mcrs_minecraft_entity_key::EntityUuid::from_u128(bits)
}

impl<'a, V: BlockVolume> CollisionWorld for SearchView<'a, V> {
    type Section<'s>
        = BlockSection<V::Section<'a>>
    where
        Self: 's;

    fn section(&self, at: SectionPos) -> SectionLookup<Self::Section<'_>> {
        self.blocks.section_at(at)
    }

    fn border(&self) -> Option<&WorldBorder> {
        None
    }
}

impl<V: BlockVolume> SearchWorld for SearchView<'_, V> {
    fn path_cell(&mut self, at: BlockPos) -> PathCell {
        match self.state_at(at) {
            Some(state) => path_cell_of(
                &self.tables.collision,
                NavRow::of(&self.tables.collision.state(state)),
            ),
            None => self.base.unloaded_cell(),
        }
    }

    fn min_y(&mut self) -> i32 {
        PathWorld::min_y(self.base)
    }

    fn max_y(&mut self) -> i32 {
        PathWorld::max_y(self.base)
    }

    fn column_loaded(&mut self, at: BlockPos) -> bool {
        PathWorld::column_loaded(self.base, at)
    }

    fn collides(&mut self, min: DVec3, max: DVec3) -> bool {
        let mut colliders = core::mem::take(&mut self.colliders);
        let view = View::new(&*self, &NoBodies, self.tables);
        let hit = intersects(view, &self.entity, Aabb::new(min, max), &mut colliders);
        self.colliders = colliders;
        hit
    }
}
