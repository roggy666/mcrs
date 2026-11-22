use bevy_ecs::lifecycle::Add;
use bevy_ecs::message::MessageReader;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::observer::On;
use bevy_ecs::query::With;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use mcrs_minecraft_damage::HitOutcome;
use mcrs_minecraft_damage::HitRecord;
use mcrs_minecraft_damage::Vitals;
use mcrs_minecraft_engine::bus::OutboundPlayerPacket;
use mcrs_minecraft_engine::bus::PacketPayload;
use mcrs_minecraft_engine::bus::to;
use mcrs_minecraft_engine::network_id::player_network_id;
use mcrs_minecraft_engine_damage::Damaged;
use mcrs_minecraft_engine_entity_motion::MotionId;
use mcrs_minecraft_engine_physics::MobIndex;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::session::Owner;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_protocol::VarInt;
use mcrs_minecraft_protocol_game::clientbound::ClientboundDamageEvent;
use mcrs_minecraft_protocol_game::clientbound::ClientboundSetHealth;

const PLAYER_HEALTH: f32 = 20.0;
const FOOD: i32 = 20;
const SATURATION: f32 = 5.0;

pub(crate) fn give_player_vitals(add: On<Add<Player>>, mut commands: Commands) {
    commands
        .entity(add.event().entity)
        .insert_if_new((Vitals::new(PLAYER_HEALTH), HitRecord::default()));
}

type HurtPlayers<'w, 's> = Query<'w, 's, (&'static Owner, &'static Vitals), With<Player>>;

pub(crate) fn report_player_hits(
    mut damaged: MessageReader<Damaged>,
    players: HurtPlayers,
    index: Res<MobIndex>,
    ids: Query<&MotionId>,
    mut packets: MessageWriter<OutboundPlayerPacket>,
) {
    for hit in damaged.read() {
        let Ok((owner, vitals)) = players.get(hit.target) else {
            continue;
        };
        let session = owner.0;
        let own_id = player_network_id(session);
        let network_id = |key: Option<EntityUuid>| -> i32 {
            key.and_then(|key| index.get(key))
                .and_then(|entity| ids.get(entity).ok())
                .map_or(0, |id| (id.0.0 as i32).saturating_add(1))
        };
        if hit.report.outcome != HitOutcome::Rejected {
            packets.write(to(
                session,
                PacketPayload::DamageEvent(ClientboundDamageEvent {
                    entity_id: VarInt(own_id),
                    damage_type: VarInt(hit.source.damage_type as i32),
                    cause_entity_id: VarInt(network_id(hit.source.cause.causing)),
                    direct_entity_id: VarInt(network_id(hit.source.cause.direct)),
                    source_position: hit.source.position,
                }),
            ));
        }
        packets.write(to(
            session,
            PacketPayload::SetHealth(ClientboundSetHealth {
                health: vitals.health,
                food: VarInt(FOOD),
                saturation: SATURATION,
            }),
        ));
    }
}
