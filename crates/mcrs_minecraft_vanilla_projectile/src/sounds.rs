use bevy_ecs::message::MessageWriter;
use glam::DVec3;
use mcrs_minecraft_engine::bus::PacketPayload;
use mcrs_minecraft_engine::column_change::ColumnChange;
use mcrs_minecraft_engine::column_streams::ColumnStreams;
use mcrs_minecraft_entity_key::KeyedStream;
use mcrs_minecraft_physics::block_at;
use mcrs_minecraft_protocol_game::clientbound::ClientboundSound;
use mcrs_minecraft_protocol_game::sound::{SoundHolder, SoundSource};
use mcrs_minecraft_sound::sound_event_keys::SoundEvent;
use mcrs_voxel::ColumnPos;

const EIGHTHS: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SoundAt {
    pub(crate) position: DVec3,
    pub(crate) sound: SoundEvent,
    pub(crate) source: SoundSource,
    pub(crate) volume: f32,
    pub(crate) pitch: f32,
}

impl SoundAt {
    pub(crate) fn arrow_hit(position: DVec3, stream: &KeyedStream) -> Self {
        let random = stream.unit(0) as f32;
        Self {
            position,
            sound: SoundEvent::EntityArrowHit,
            source: SoundSource::Neutral,
            volume: 1.0,
            pitch: 1.2 / (random * 0.2 + 0.9),
        }
    }

    pub(crate) fn arrow_shoot(
        position: DVec3,
        source: SoundSource,
        power: f32,
        stream: &KeyedStream,
    ) -> Self {
        let random = stream.unit(0) as f32;
        Self {
            position,
            sound: SoundEvent::EntityArrowShoot,
            source,
            volume: 1.0,
            pitch: 1.0 / (random * 0.4 + 1.2) + power * 0.5,
        }
    }

    pub(crate) fn snowball_throw(position: DVec3, stream: &KeyedStream) -> Self {
        let random = stream.unit(0) as f32;
        Self {
            position,
            sound: SoundEvent::EntitySnowballThrow,
            source: SoundSource::Neutral,
            volume: 0.5,
            pitch: 0.4 / (random * 0.4 + 0.8),
        }
    }
}

pub(crate) fn play(
    sound: SoundAt,
    seed: u64,
    streams: Option<&ColumnStreams>,
    changes: &mut MessageWriter<ColumnChange>,
) {
    let column = ColumnPos::from(block_at(sound.position));
    if !streams.is_some_and(|streams| streams.is_subscribed(column)) {
        return;
    }
    let eighths = |value: f64| (value * EIGHTHS).round() as i32;
    changes.write(ColumnChange {
        column,
        payload: PacketPayload::Sound(ClientboundSound {
            sound: SoundHolder::Registry(sound.sound.id().number()),
            source: sound.source,
            x: eighths(sound.position.x),
            y: eighths(sound.position.y),
            z: eighths(sound.position.z),
            volume: sound.volume,
            pitch: sound.pitch,
            seed: seed as i64,
        }),
        except: None,
    });
}
