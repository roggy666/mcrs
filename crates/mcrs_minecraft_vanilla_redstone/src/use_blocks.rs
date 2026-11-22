use bevy_ecs::entity::Entity;
use bevy_ecs::message::{MessageReader, MessageWriter};
use bevy_ecs::query::With;
use bevy_ecs::system::{Local, Query, Res};
use mcrs_minecraft_dimension::Dimension as DimensionKey;
use mcrs_minecraft_engine::block_simulation::{BlockRules, block_sound};
use mcrs_minecraft_engine::bus::PacketPayload;
use mcrs_minecraft_engine::column_change::ColumnChange;
use mcrs_minecraft_engine::coordinator::clock::GameTime;
use mcrs_minecraft_engine::teleport::TeleportState;
use mcrs_minecraft_engine::use_item_on_request::{UseItemOnConsumed, UseItemOnRequest};
use mcrs_minecraft_engine::world_kinds::WorldKinds;
use mcrs_minecraft_engine::{PlayerBody, ShiftKey, eye_position, in_reach, reach};
use mcrs_minecraft_engine_item::{Selected, Slots};
use mcrs_minecraft_engine_world::block_update::{BlockSetRequest, remove_block};
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::world::dimension::InDimension;
use mcrs_minecraft_engine_world::world::storage::view::WorldView;
use mcrs_minecraft_engine_world::world_seed::WorldSeed;
use mcrs_minecraft_game_mode::{GameMode, PlayerGameMode};
use mcrs_minecraft_inventory_player::OFF_HAND;
use mcrs_minecraft_random::Random;
use mcrs_minecraft_redstone::{BlockUse, Click, HasRedstone};
use mcrs_minecraft_resource::ResourceKeyBuf;
use mcrs_minecraft_simulation::{BlockBehaviour, GameTick, StreamKey, StreamSeed};
use mcrs_minecraft_volume::BlockVolume;
use mcrs_voxel::{BlockPos, BlockStateId, ColumnPos};
use rustc_hash::FxHashMap;

type UserItem<'a> = (
    &'a InDimension,
    &'a Transform,
    &'a PlayerGameMode,
    PlayerBody,
    Option<&'a ShiftKey>,
    Option<&'a TeleportState>,
    Option<&'a Slots>,
    Option<&'a Selected>,
);

fn holds_an_item(slots: Option<&Slots>, selected: Option<&Selected>) -> bool {
    let Some(slots) = slots else {
        return false;
    };
    let main = selected.and_then(|selected| slots.get(usize::from(selected.0)));
    main.is_some() || slots.get(usize::from(OFF_HAND)).is_some()
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each resource, query, reader and writer is its own parameter"
)]
pub(crate) fn use_redstone_blocks<B>(
    mut requests: MessageReader<UseItemOnRequest>,
    users: Query<UserItem, With<Player>>,
    dimensions: Query<Option<&ResourceKeyBuf<DimensionKey>>>,
    world: WorldView,
    rules: Res<BlockRules<B>>,
    time: Res<GameTime>,
    seed: Option<Res<WorldSeed>>,
    mut blocks: MessageWriter<BlockSetRequest>,
    mut consumed: MessageWriter<UseItemOnConsumed>,
    mut changes: MessageWriter<ColumnChange>,
    mut pending: Local<FxHashMap<(Entity, BlockPos), BlockStateId>>,
) where
    B: BlockBehaviour<WorldKinds> + HasRedstone + Send + Sync + 'static,
{
    let redstone = rules.0.redstone();
    pending.clear();
    for request in requests.read() {
        if request.sequence.0 < 0 {
            continue;
        }
        let Ok((dimension, transform, mode, body, shift, teleport, slots, selected)) =
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
        if shift.is_some_and(|shift| shift.0) && holds_an_item(slots, selected) {
            continue;
        }
        let eye = eye_position(transform.translation, body.dimensions());
        if !reach(body.attributes).is_some_and(|limit| in_reach(eye, request.block_pos, limit)) {
            continue;
        }
        let Some(view) = world.get(dimension.0) else {
            continue;
        };
        let key = (dimension.0, request.block_pos);
        let state = match pending.get(&key) {
            Some(&state) => state,
            None => match view.block(request.block_pos) {
                Ok(state) => state,
                Err(_) => continue,
            },
        };
        match redstone.used(state) {
            BlockUse::Toggle(toggle) => {
                pending.insert(key, toggle.state);
                blocks.write(BlockSetRequest {
                    new_state: toggle.state,
                    flags: toggle.flags,
                    ..remove_block(dimension.0, request.block_pos)
                });
                if let Some(click) = toggle.sound {
                    let now = GameTick(time.0);
                    let world_seed = seed.as_deref().map_or(0, |seed| seed.0);
                    let dimension_key = dimensions
                        .get(dimension.0)
                        .ok()
                        .flatten()
                        .map_or("", |key| key.as_str());
                    let stream = StreamSeed::new(world_seed as i64, dimension_key);
                    changes.write(use_sound(request.block_pos, click, now, &stream));
                }
            }
            BlockUse::Consumed => {}
            BlockUse::Unhandled => continue,
        }
        consumed.write(UseItemOnConsumed {
            entity: request.entity,
            sequence: request.sequence,
        });
    }
}

fn use_sound(pos: BlockPos, click: Click, now: GameTick, seed: &StreamSeed) -> ColumnChange {
    let pitch = if click.pitch.min == click.pitch.max {
        click.pitch.min
    } else {
        let unit = seed
            .stream(now, StreamKey::Effect { pos, index: 0 })
            .next_f64();
        (f64::from(click.pitch.min) + unit * f64::from(click.pitch.max - click.pitch.min)) as f32
    };
    ColumnChange {
        column: ColumnPos::from(pos),
        payload: PacketPayload::Sound(block_sound(pos, click.sound, click.volume, pitch, now)),
        except: None,
    }
}
