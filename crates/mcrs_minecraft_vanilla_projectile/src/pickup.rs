use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::Remove;
use bevy_ecs::observer::On;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use glam::DVec3;
use mcrs_engine_muxer::EntityEvent;
use mcrs_minecraft_engine::tile_events::TileEvents;
use mcrs_minecraft_engine_entity_motion::MotionId;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_entity_motion_replication::PublishedTile;
use mcrs_minecraft_engine_item::HolderCommandsExt;
use mcrs_minecraft_engine_physics::PlayerIndex;
use mcrs_minecraft_item_key::item_keys::Item;
use mcrs_minecraft_item_stack::Count;
use mcrs_minecraft_item_stack::ItemStack;
use mcrs_minecraft_item_stack::Kind;
use mcrs_minecraft_physics::Aabb;
use mcrs_minecraft_physics::block_at;
use mcrs_minecraft_player_index::PlayerFlags;
use mcrs_minecraft_replication::MobReport;
use mcrs_minecraft_replication::MobSignal;
use mcrs_voxel::ColumnPos;
use rustc_hash::FxHashMap;

use crate::Projectile;
use crate::projectile::Pickup;

const PICKUP_REACH: DVec3 = DVec3::new(1.0, 0.5, 1.0);
const ARROW_SIZE: f32 = 0.5;

#[derive(Resource, Debug, Default)]
pub struct RestingArrows {
    columns: FxHashMap<ColumnPos, Vec<Entity>>,
    resting: FxHashMap<Entity, ColumnPos>,
}

impl RestingArrows {
    pub fn rest(&mut self, arrow: Entity, position: DVec3) {
        let column = ColumnPos::from(block_at(position));
        if self.resting.insert(arrow, column) == Some(column) {
            return;
        }
        self.columns.entry(column).or_default().push(arrow);
    }

    pub fn leave(&mut self, arrow: Entity) {
        let Some(column) = self.resting.remove(&arrow) else {
            return;
        };
        if let Some(arrows) = self.columns.get_mut(&column) {
            arrows.retain(|resting| *resting != arrow);
            if arrows.is_empty() {
                self.columns.remove(&column);
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.resting.is_empty()
    }

    pub fn around(&self, column: ColumnPos) -> impl Iterator<Item = Entity> + '_ {
        (-1..=1).flat_map(move |dx| {
            (-1..=1).flat_map(move |dz| {
                self.columns
                    .get(&ColumnPos::new(column.x + dx, column.z + dz))
                    .into_iter()
                    .flatten()
                    .copied()
            })
        })
    }
}

pub(crate) fn forget_resting(removed: On<Remove<Projectile>>, mut resting: ResMut<RestingArrows>) {
    resting.leave(removed.event().entity);
}

pub(crate) fn has_resting_arrows(resting: Res<RestingArrows>) -> bool {
    !resting.is_empty()
}

type Arrows<'w, 's> = Query<
    'w,
    's,
    (
        &'static Projectile,
        &'static MotionState,
        Option<&'static MotionId>,
        Option<&'static PublishedTile>,
    ),
>;

pub(crate) fn pick_up_arrows(
    collectors: Res<PlayerIndex>,
    arrows: Arrows,
    mut resting: ResMut<RestingArrows>,
    mut events: ResMut<TileEvents>,
    mut commands: Commands,
) {
    let arrow_item = Kind(Item::Arrow.id().number());
    let mut taken: Vec<Entity> = Vec::new();
    for collector in collectors.rows() {
        if collector.flags.contains(PlayerFlags::SPECTATOR) {
            continue;
        }
        let Some(player) = collectors.entity_of(collector.player) else {
            continue;
        };
        let reach = Aabb::new(collector.min(), collector.max()).inflate(PICKUP_REACH);
        let column = ColumnPos::from(block_at(collector.position));
        for arrow in resting.around(column) {
            if taken.contains(&arrow) {
                continue;
            }
            let Ok((projectile, state, id, published)) = arrows.get(arrow) else {
                continue;
            };
            if !projectile.in_ground || projectile.shake > 0 {
                continue;
            }
            let takes = match projectile.pickup {
                Pickup::Disallowed => false,
                Pickup::Allowed => true,
                Pickup::CreativeOnly => collector.flags.contains(PlayerFlags::CREATIVE),
            };
            if !takes {
                continue;
            }
            let row = Aabb::of_body(state.0.body.position, ARROW_SIZE, ARROW_SIZE);
            if !reach.intersects(&row) {
                continue;
            }
            if projectile.pickup == Pickup::Allowed
                && let Some(count) = Count::new(1)
            {
                commands
                    .entity(player)
                    .give(ItemStack::new(arrow_item, count));
            }
            if let (Some(id), Some(tile)) = (id, published.and_then(|published| published.tile())) {
                events.push(
                    tile,
                    id.0,
                    EntityEvent::Data(
                        MobReport::Signal(MobSignal::Taken {
                            collector: collector.player,
                            amount: 1,
                        })
                        .into(),
                    ),
                );
            }
            taken.push(arrow);
        }
    }
    for arrow in taken {
        resting.leave(arrow);
        commands.entity(arrow).despawn();
    }
}
