use bevy_app::{App, FixedUpdate, Plugin};
use bevy_ecs::message::{MessageCursor, MessageReader, MessageWriter, Messages};
use bevy_ecs::query::With;
use bevy_ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy_ecs::system::{Commands, Local, Query, Res, ResMut};
use glam::DVec3;
use mcrs_minecraft_block::definition::{BlockStateFlags, Blocks};
use mcrs_minecraft_engine::swing::SwingPlayer;
use mcrs_minecraft_engine::teleport::TeleportState;
use mcrs_minecraft_engine::use_item_on_request::{
    BlockUseSet, UseItemOnConsumed, UseItemOnRequest,
};
use mcrs_minecraft_engine::{BlockBreakSet, PlayerBody, eye_position, in_reach, reach};
use mcrs_minecraft_engine_item::{Selected, Slots, StackCommandsExt};
use mcrs_minecraft_engine_physics::{MobExtras, MobSpawnRequest, SpawnSetup};
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::entity_births::EntityBirths;
use mcrs_minecraft_engine_world::tick::TickSet;
use mcrs_minecraft_engine_world::world::dimension::InDimension;
use mcrs_minecraft_engine_world::world::storage::view::WorldView;
use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_entity_key::{BirthSequences, BirthSource};
use mcrs_minecraft_game_mode::{GameMode, PlayerGameMode};
use mcrs_minecraft_inventory_player::OFF_HAND;
use mcrs_minecraft_item::Items;
use mcrs_minecraft_item::component::EntityData;
use mcrs_minecraft_item_stack::{InteractAnimation, Kind};
use mcrs_minecraft_protocol_game::Hand;
use mcrs_minecraft_resource::Id;
use mcrs_minecraft_volume::BlockVolume;

use crate::block_place_plugin::placement_swing;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpawnEggSet;

#[derive(Clone, Copy, Debug, Default)]
pub struct SpawnEggPlugin;

impl Plugin for SpawnEggPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MobSpawnRequest>()
            .add_message::<SwingPlayer>()
            .add_systems(
                FixedUpdate,
                use_spawn_eggs
                    .in_set(SpawnEggSet)
                    .in_set(TickSet::ClientInput)
                    .after(BlockBreakSet)
                    .after(BlockUseSet),
            );
    }
}

type UserItem<'a> = (
    &'a InDimension,
    &'a Transform,
    &'a PlayerGameMode,
    PlayerBody,
    Option<&'a TeleportState>,
    Option<&'a Slots>,
    Option<&'a Selected>,
);

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each query, writer and resource is its own parameter"
)]
fn use_spawn_eggs(
    mut requests: MessageReader<UseItemOnRequest>,
    mut consumed: ResMut<Messages<UseItemOnConsumed>>,
    mut cursor: Local<MessageCursor<UseItemOnConsumed>>,
    users: Query<UserItem, With<Player>>,
    stacks: Query<(&Kind, Option<&InteractAnimation>)>,
    world: WorldView,
    definitions: Option<Res<Blocks>>,
    items: Option<Res<Items>>,
    births: EntityBirths,
    mut sequences: Local<BirthSequences>,
    mut spawns: MessageWriter<MobSpawnRequest>,
    mut swings: MessageWriter<SwingPlayer>,
    mut commands: Commands,
) {
    let (Some(definitions), Some(items)) = (definitions, items) else {
        requests.clear();
        return;
    };
    let taken: Vec<UseItemOnConsumed> = cursor.read(&consumed).copied().collect();
    for request in requests.read() {
        if request.sequence.0 < 0 {
            continue;
        }
        if taken
            .iter()
            .any(|used| used.entity == request.entity && used.sequence == request.sequence)
        {
            continue;
        }
        let Ok((dimension, transform, mode, body, teleport, slots, selected)) =
            users.get(request.entity)
        else {
            continue;
        };
        if mode.0 == GameMode::Spectator {
            continue;
        }
        if teleport.is_some_and(|teleport| teleport.pending().is_some()) {
            continue;
        }
        let eye = eye_position(transform.translation, body.dimensions());
        if !reach(body.attributes).is_some_and(|limit| in_reach(eye, request.block_pos, limit)) {
            continue;
        }
        let slot = match request.hand {
            Hand::Main => selected.map(|selected| usize::from(selected.0)),
            Hand::Off => Some(usize::from(OFF_HAND)),
        };
        let Some(stack) = slot.and_then(|slot| slots?.get(slot)) else {
            continue;
        };
        let Ok((kind, interact)) = stacks.get(stack) else {
            continue;
        };
        let Some(mob) = egg_mob(&items, *kind) else {
            continue;
        };
        let Some(view) = world.get(dimension.0) else {
            continue;
        };
        let Ok(clicked) = view.block(request.block_pos) else {
            continue;
        };
        let replaceable = definitions
            .state(clicked)
            .flags
            .contains(BlockStateFlags::REPLACEABLE);
        let target = if replaceable {
            request.block_pos
        } else {
            request.block_pos + request.face.normal()
        };
        let birth = BirthSource::Block(target);
        let uuid = births.born(&birth, sequences.next(births.tick(), &birth));
        spawns.write(MobSpawnRequest {
            kind: mob,
            position: DVec3::new(
                f64::from(target.x) + 0.5,
                f64::from(target.y),
                f64::from(target.z) + 0.5,
            ),
            uuid: uuid.as_u128(),
            setup: SpawnSetup::First,
            extras: MobExtras::default(),
        });
        if mode.0 != GameMode::Creative {
            commands.entity(stack).consume(1);
        }
        swings.write(placement_swing(
            request.entity,
            request.hand,
            interact.map(|animation| animation.0),
        ));
        consumed.write(UseItemOnConsumed {
            entity: request.entity,
            sequence: request.sequence,
        });
    }
}

fn egg_mob(items: &Items, kind: Kind) -> Option<&'static EntityTypeData> {
    let entry = items.get(Id::checked(kind.0, items.len())?)?;
    let data = entry.prototype.get::<EntityData>()?;
    let entity_type = EntityType::get(data.0.id.location().as_location())?;
    EntityTypeData::of(entity_type).filter(|kind| kind.mob().is_some())
}
