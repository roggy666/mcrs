use bevy_ecs::entity::Entity;
use bevy_ecs::message::{Message, MessageWriter};
use bevy_ecs::observer::On;
use mcrs_minecraft_core::Look;
use mcrs_minecraft_network::event::Received;
use mcrs_minecraft_protocol::VarInt;
use mcrs_minecraft_protocol_game::Hand;
use mcrs_minecraft_protocol_game::serverbound::ServerboundUseItem;

#[derive(Message, Clone, Copy, Debug)]
pub struct UseItemRequest {
    pub entity: Entity,
    pub hand: Hand,
    pub sequence: VarInt,
    pub look: Look,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct ReleaseItemRequest {
    pub entity: Entity,
    pub sequence: VarInt,
}

pub(crate) fn handle_use_item(
    on: On<Received<ServerboundUseItem>>,
    mut writer: MessageWriter<UseItemRequest>,
) {
    let Some(p) = on.packet() else {
        return;
    };
    writer.write(UseItemRequest {
        entity: on.entity,
        hand: p.hand,
        sequence: p.sequence,
        look: p.look,
    });
}
