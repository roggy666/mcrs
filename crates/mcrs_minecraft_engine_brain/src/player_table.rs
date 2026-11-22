use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::With;
use bevy_ecs::system::Query;
use mcrs_minecraft_brain::HurtBy;
use mcrs_minecraft_brain::ItemKind;
use mcrs_minecraft_brain::PlayerFlags;
use mcrs_minecraft_brain::PlayerRow;
use mcrs_minecraft_brain::PlayerTable;
use mcrs_minecraft_engine::PlayerBody;
use mcrs_minecraft_engine::network_id::player_network_id;
use mcrs_minecraft_engine_item::Selected;
use mcrs_minecraft_engine_item::Slots;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::session::Owner;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_game_mode::GameMode;
use mcrs_minecraft_game_mode::PlayerGameMode;
use mcrs_minecraft_inventory_player::OFF_HAND;
use mcrs_minecraft_item_stack::Kind;

pub(crate) type Players<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Owner,
        &'static EntityUuid,
        &'static Transform,
        Option<&'static PlayerGameMode>,
        PlayerBody,
        Option<&'static Selected>,
        Option<&'static Slots>,
    ),
    With<Player>,
>;

pub(crate) type Stacks<'w, 's> = Query<'w, 's, &'static Kind>;

fn hand(slots: Option<&Slots>, slot: Option<usize>, stacks: &Stacks<'_, '_>) -> ItemKind {
    slots
        .zip(slot)
        .and_then(|(slots, slot)| slots.get(slot))
        .and_then(|stack| stacks.get(stack).ok())
        .map_or(ItemKind::EMPTY, |kind| ItemKind(kind.0))
}

pub(crate) fn player_table(players: &Players<'_, '_>, stacks: &Stacks<'_, '_>) -> PlayerTable {
    PlayerTable::new(players.iter().map(
        |(_, owner, uuid, transform, mode, body, selected, slots)| {
            let mode = mode.map(|mode| mode.0);
            let dimensions = body.dimensions();
            let marks = [
                (true, PlayerFlags::ALIVE),
                (mode == Some(GameMode::Spectator), PlayerFlags::SPECTATOR),
                (mode == Some(GameMode::Creative), PlayerFlags::CREATIVE),
            ];
            let flags = marks
                .into_iter()
                .filter(|(set, _)| *set)
                .fold(PlayerFlags::NONE, |flags, (_, flag)| flags.union(flag));
            PlayerRow {
                player: player_id(owner),
                key: *uuid,
                position: transform.translation,
                width: dimensions.width,
                height: dimensions.height,
                eye_height: dimensions.eye_height,
                flags,
                visibility: 1.0,
                vehicle: None,
                hands: [
                    hand(
                        slots,
                        selected.map(|selected| usize::from(selected.0)),
                        stacks,
                    ),
                    hand(slots, Some(usize::from(OFF_HAND)), stacks),
                ],
                hurt_by: HurtBy::NONE,
            }
        },
    ))
}

fn player_id(owner: &Owner) -> EntityId {
    EntityId(u64::from(player_network_id(owner.0).cast_unsigned()))
}

pub(crate) fn player_entities(players: &Players<'_, '_>) -> Vec<(EntityId, Entity)> {
    let mut entities: Vec<(EntityId, Entity)> = players
        .iter()
        .map(|(entity, owner, ..)| (player_id(owner), entity))
        .collect();
    entities.sort_unstable_by_key(|(id, _)| *id);
    entities
}
