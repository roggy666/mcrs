use bevy_ecs::component::Component;
use bevy_ecs::query::Changed;
use bevy_ecs::query::With;
use bevy_ecs::system::Query;
use bevy_ecs::system::ResMut;
use mcrs_engine_muxer::EntityEvent;
use mcrs_minecraft_damage::Fire;
use mcrs_minecraft_engine::tile_events::TileEvents;
use mcrs_minecraft_engine_entity_motion::MotionId;
use mcrs_minecraft_engine_entity_motion_replication::PublishedTile;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_protocol_game::entity::MetaDataValue;
use mcrs_minecraft_protocol_game::entity::MetadataEntry;
use mcrs_minecraft_protocol_game::entity_data_index::SHARED_FLAG_ON_FIRE;
use mcrs_minecraft_protocol_game::entity_data_index::SHARED_FLAGS;
use mcrs_minecraft_replication::MobReport;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BurningShown(pub bool);

type Mobs<'w, 's> = Query<
    'w,
    's,
    (
        &'static Fire,
        &'static MotionId,
        Option<&'static PublishedTile>,
        &'static mut BurningShown,
    ),
    (With<Mob>, Changed<Fire>),
>;

pub(crate) fn publish_burning(mut mobs: Mobs, mut events: ResMut<TileEvents>) {
    for (fire, id, published, mut shown) in &mut mobs {
        let burning = fire.burning();
        if shown.0 == burning {
            continue;
        }
        shown.0 = burning;
        let Some(tile) = published.and_then(|published| published.tile()) else {
            continue;
        };
        let flags = if burning { SHARED_FLAG_ON_FIRE } else { 0 };
        events.push(
            tile,
            id.0,
            EntityEvent::Data(
                MobReport::Entries(vec![MetadataEntry {
                    index: SHARED_FLAGS,
                    value: MetaDataValue::Byte(flags.cast_signed()),
                }])
                .into(),
            ),
        );
    }
}
