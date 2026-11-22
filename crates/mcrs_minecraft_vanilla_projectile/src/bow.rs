use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageReader;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::query::With;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use glam::DVec3;
use mcrs_minecraft_engine::PlayerBody;
use mcrs_minecraft_engine::eye_position;
use mcrs_minecraft_engine::use_item_request::ReleaseItemRequest;
use mcrs_minecraft_engine::use_item_request::UseItemRequest;
use mcrs_minecraft_engine_item::Selected;
use mcrs_minecraft_engine_item::Slots;
use mcrs_minecraft_engine_item::StackCommandsExt;
use mcrs_minecraft_engine_physics::PlayerIndex;
use mcrs_minecraft_engine_world::birth_clock::BirthClock;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::session::Owner;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::KeyedStream;
use mcrs_minecraft_game_mode::GameMode;
use mcrs_minecraft_game_mode::PlayerGameMode;
use mcrs_minecraft_inventory_player::OFF_HAND;
use mcrs_minecraft_item_key::item_keys::Item;
use mcrs_minecraft_item_stack::Kind;
use mcrs_minecraft_player_index::PlayerFlags;
use mcrs_minecraft_protocol_game::Hand;
use mcrs_minecraft_protocol_game::sound::SoundSource;

use crate::ProjectileKind;
use crate::aim::inherited_motion;
use crate::aim::launch_velocity;
use crate::launch::LaunchSound;
use crate::launch::ProjectileLaunch;
use crate::launch::player_id;
use crate::projectile::Pickup;

const START_BELOW_EYE: f64 = 0.1;
const BOW_SPEED: f64 = 3.0;
const SNOWBALL_SPEED: f64 = 1.5;
const INACCURACY: f64 = 1.0;
const LEAST_POWER: f32 = 0.1;
const FULL_DRAW: f32 = 20.0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawing {
    pub hand: Hand,
    pub ticks: u64,
}

#[must_use]
pub fn bow_power(draw_ticks: u64) -> f32 {
    let draw = draw_ticks.min(u64::from(u16::MAX)) as f32 / FULL_DRAW;
    ((draw * draw + draw * 2.0) / 3.0).min(1.0)
}

type Users<'w, 's> = Query<
    'w,
    's,
    (
        &'static Owner,
        &'static EntityUuid,
        &'static Transform,
        &'static PlayerGameMode,
        PlayerBody,
        Option<&'static Slots>,
        Option<&'static Selected>,
        Option<&'static Drawing>,
    ),
    With<Player>,
>;

fn hand_slot(hand: Hand, selected: Option<&Selected>) -> Option<usize> {
    match hand {
        Hand::Main => Some(selected.map_or(0, |selected| usize::from(selected.0))),
        Hand::Off => Some(usize::from(OFF_HAND)),
    }
}

fn kind_of(stacks: &Query<&Kind>, slots: Option<&Slots>, slot: Option<usize>) -> Option<Kind> {
    let stack = slots?.get(slot?)?;
    stacks.get(stack).ok().copied()
}

fn direction(yaw: f32, pitch: f32) -> DVec3 {
    let (yaw, pitch) = (yaw.to_radians(), pitch.to_radians());
    DVec3::new(
        f64::from(-yaw.sin() * pitch.cos()),
        f64::from(-pitch.sin()),
        f64::from(yaw.cos() * pitch.cos()),
    )
}

pub(crate) fn start_item_use(
    mut requests: MessageReader<UseItemRequest>,
    users: Users,
    stacks: Query<&Kind>,
    clock: Res<BirthClock>,
    players: Res<PlayerIndex>,
    mut launches: MessageWriter<ProjectileLaunch>,
    mut commands: Commands,
) {
    let bow = Kind(Item::Bow.id().number());
    let snowball = Kind(Item::Snowball.id().number());
    for request in requests.read() {
        let Ok((owner, uuid, transform, mode, body, slots, selected, drawing)) =
            users.get(request.entity)
        else {
            continue;
        };
        if mode.0 == GameMode::Spectator {
            continue;
        }
        let slot = hand_slot(request.hand, selected);
        let Some(kind) = kind_of(&stacks, slots, slot) else {
            continue;
        };
        if kind == bow {
            if drawing.is_none() {
                commands.entity(request.entity).insert(Drawing {
                    hand: request.hand,
                    ticks: 0,
                });
            }
            continue;
        }
        if kind != snowball {
            continue;
        }
        let feet = transform.translation;
        let position =
            eye_position(feet, body.dimensions()) - DVec3::new(0.0, START_BELOW_EYE, 0.0);
        let stream = KeyedStream::new(uuid, clock.0);
        let velocity = launch_velocity(
            direction(request.look.yaw, request.look.pitch),
            SNOWBALL_SPEED,
            INACCURACY,
            &stream,
        ) + shooter_motion(&players, player_id(owner));
        launches.write(ProjectileLaunch {
            kind: ProjectileKind::Snowball,
            owner: Some((*uuid, player_id(owner))),
            position,
            velocity,
            critical: false,
            pickup: Pickup::Disallowed,
            sound: Some(LaunchSound::Throw),
        });
        if mode.0 != GameMode::Creative
            && let Some(stack) = slots.and_then(|slots| slots.get(slot?))
        {
            commands.entity(stack).consume(1);
        }
    }
}

pub(crate) fn count_draws(mut drawings: Query<&mut Drawing>) {
    for mut drawing in &mut drawings {
        drawing.ticks = drawing.ticks.saturating_add(1);
    }
}

pub(crate) fn release_item_use(
    mut releases: MessageReader<ReleaseItemRequest>,
    users: Users,
    stacks: Query<&Kind>,
    clock: Res<BirthClock>,
    players: Res<PlayerIndex>,
    mut launches: MessageWriter<ProjectileLaunch>,
    mut commands: Commands,
) {
    let bow = Kind(Item::Bow.id().number());
    let arrow = Kind(Item::Arrow.id().number());
    for release in releases.read() {
        let Ok((owner, uuid, transform, mode, body, slots, selected, drawing)) =
            users.get(release.entity)
        else {
            continue;
        };
        let Some(drawing) = drawing else {
            continue;
        };
        commands.entity(release.entity).remove::<Drawing>();
        if kind_of(&stacks, slots, hand_slot(drawing.hand, selected)) != Some(bow) {
            continue;
        }
        let power = bow_power(drawing.ticks);
        if power < LEAST_POWER {
            continue;
        }
        let creative = mode.0 == GameMode::Creative;
        let ammo = arrow_stack(&stacks, slots, drawing.hand, selected, arrow);
        if ammo.is_none() && !creative {
            continue;
        }
        let feet = transform.translation;
        let position =
            eye_position(feet, body.dimensions()) - DVec3::new(0.0, START_BELOW_EYE, 0.0);
        let stream = KeyedStream::new(uuid, clock.0);
        let velocity = launch_velocity(
            direction(transform.rotation.yaw(), transform.rotation.pitch()),
            BOW_SPEED * f64::from(power),
            INACCURACY,
            &stream,
        ) + shooter_motion(&players, player_id(owner));
        launches.write(ProjectileLaunch {
            kind: ProjectileKind::Arrow,
            owner: Some((*uuid, player_id(owner))),
            position,
            velocity,
            critical: power >= 1.0,
            pickup: if creative {
                Pickup::CreativeOnly
            } else {
                Pickup::Allowed
            },
            sound: Some(LaunchSound::Bow {
                source: SoundSource::Players,
                power,
            }),
        });
        if let (Some(stack), false) = (ammo, creative) {
            commands.entity(stack).consume(1);
        }
    }
}

fn arrow_stack(
    stacks: &Query<&Kind>,
    slots: Option<&Slots>,
    bow_hand: Hand,
    selected: Option<&Selected>,
    arrow: Kind,
) -> Option<Entity> {
    let slots = slots?;
    let other_hand = match bow_hand {
        Hand::Main => Some(usize::from(OFF_HAND)),
        Hand::Off => Some(selected.map_or(0, |selected| usize::from(selected.0))),
    };
    let holds_arrow = |stack: Entity| stacks.get(stack).ok().copied() == Some(arrow);
    if let Some(stack) = other_hand.and_then(|slot| slots.get(slot))
        && holds_arrow(stack)
    {
        return Some(stack);
    }
    slots.occupied().find(|stack| holds_arrow(*stack))
}

/// The motion of a player that its projectile inherits (PRJ-LCH-004): the movement of the last
/// movement packet, which the player index keeps, or zero for a player that the index lacks.
fn shooter_motion(players: &PlayerIndex, player: EntityId) -> DVec3 {
    players.find(player).map_or(DVec3::ZERO, |row| {
        inherited_motion(
            row.motion.as_dvec3(),
            row.flags.contains(PlayerFlags::ON_GROUND),
        )
    })
}
