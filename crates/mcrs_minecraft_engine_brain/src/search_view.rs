use std::cell::RefCell;
use std::rc::Rc;

use glam::DVec3;
use mcrs_minecraft_collision_table::BlockSection;
use mcrs_minecraft_collision_table::BlockWorld;
use mcrs_minecraft_pathfind::MobPath;
use mcrs_minecraft_pathfind::PathCell;
use mcrs_minecraft_pathfind::PathWorld;
use mcrs_minecraft_physics::Aabb;
use mcrs_minecraft_physics::CollisionSection;
use mcrs_minecraft_physics::CollisionWorld;
use mcrs_minecraft_physics::EntityCollider;
use mcrs_minecraft_physics::NoBodies;
use mcrs_minecraft_physics::QueryEntity;
use mcrs_minecraft_physics::SectionLookup;
use mcrs_minecraft_physics::SectionMasks;
use mcrs_minecraft_physics::Tables;
use mcrs_minecraft_physics::View;
use mcrs_minecraft_physics::WorldBorder;
use mcrs_minecraft_physics::intersects;
use mcrs_minecraft_volume::BlockVolume;
use mcrs_voxel::BlockPos;
use mcrs_voxel::BlockStateId;
use mcrs_voxel::LocalPos;
use mcrs_voxel::SectionPos;
use rustc_hash::FxHashMap;

use crate::BlockGoalWorld;
use crate::block_goal_world::path_cell_of;

const CACHE_LIMIT: usize = 64;

pub struct SearchView<'a, V: BlockVolume> {
    base: &'a BlockGoalWorld<'a, V>,
    blocks: BlockWorld<'a, V>,
    tables: &'a Tables,
    entity: QueryEntity,
    cache: RefCell<FxHashMap<SectionPos, SectionLookup<Rc<BlockSection<V::Section<'a>>>>>>,
    colliders: RefCell<Vec<EntityCollider>>,
}

pub struct Shared<S>(Rc<S>);

impl<S: CollisionSection> CollisionSection for Shared<S> {
    fn masks(&self) -> Option<&SectionMasks> {
        self.0.masks()
    }

    fn state(&self, at: LocalPos) -> BlockStateId {
        self.0.state(at)
    }
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
            cache: RefCell::new(FxHashMap::default()),
            colliders: RefCell::new(Vec::new()),
        }
    }

    fn state_at(&self, at: BlockPos) -> Option<BlockStateId> {
        match self.section(SectionPos::from(at)) {
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
        = Shared<BlockSection<V::Section<'a>>>
    where
        Self: 's;

    fn section(&self, at: SectionPos) -> SectionLookup<Self::Section<'_>> {
        let mut cache = self.cache.borrow_mut();
        if let Some(found) = cache.get(&at) {
            return match found {
                SectionLookup::Loaded(section) => SectionLookup::Loaded(Shared(Rc::clone(section))),
                SectionLookup::Outside => SectionLookup::Outside,
                SectionLookup::Unloaded => SectionLookup::Unloaded,
            };
        }
        let looked = match self.blocks.section_at(at) {
            SectionLookup::Loaded(section) => SectionLookup::Loaded(Rc::new(section)),
            SectionLookup::Outside => SectionLookup::Outside,
            SectionLookup::Unloaded => SectionLookup::Unloaded,
        };
        let out = match &looked {
            SectionLookup::Loaded(section) => SectionLookup::Loaded(Shared(Rc::clone(section))),
            SectionLookup::Outside => SectionLookup::Outside,
            SectionLookup::Unloaded => SectionLookup::Unloaded,
        };
        if cache.len() < CACHE_LIMIT {
            cache.insert(at, looked);
        }
        out
    }

    fn border(&self) -> Option<&WorldBorder> {
        None
    }
}

impl<V: BlockVolume> PathWorld for SearchView<'_, V> {
    fn path_cell(&self, at: BlockPos) -> PathCell {
        match self.state_at(at) {
            Some(state) => path_cell_of(&self.tables.collision, self.tables.collision.state(state)),
            None => self.base.unloaded_cell(),
        }
    }

    fn min_y(&self) -> i32 {
        self.base.min_y()
    }

    fn max_y(&self) -> i32 {
        self.base.max_y()
    }

    fn column_loaded(&self, at: BlockPos) -> bool {
        matches!(self.section(SectionPos::from(at)), SectionLookup::Loaded(_))
    }

    fn collides(&self, min: DVec3, max: DVec3) -> bool {
        let view = View::new(self, &NoBodies, self.tables);
        let mut colliders = self.colliders.borrow_mut();
        intersects(view, &self.entity, Aabb::new(min, max), &mut colliders)
    }
}
