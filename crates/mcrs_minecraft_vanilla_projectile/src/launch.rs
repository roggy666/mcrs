use bevy_ecs::message::Message;
use bevy_ecs::message::MessageReader;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::query::With;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Local;
use bevy_ecs::system::Query;
use glam::DVec3;
use mcrs_minecraft_engine::PlayerBody;
use mcrs_minecraft_engine::network_id::player_network_id;
use mcrs_minecraft_engine_brain::RangedAttack;
use mcrs_minecraft_engine_entity_motion::MotionId;
use mcrs_minecraft_engine_entity_motion::MotionKind;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_entity_motion_replication::Appearance;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::entity_births::EntityBirths;
use mcrs_minecraft_engine_world::session::Owner;
use mcrs_minecraft_entity_key::BirthSequences;
use mcrs_minecraft_entity_key::BirthSource;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::KeyedStream;
use mcrs_minecraft_physics::Body;
use mcrs_minecraft_physics::EntityState;
use mcrs_minecraft_physics::block_at;
use mcrs_minecraft_replication::SpawnLook;

use crate::Projectile;
use crate::ProjectileKind;
use crate::aim::Aimed;
use crate::aim::Shooter;
use crate::aim::mob_shot;

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct ProjectileLaunch {
    pub kind: ProjectileKind,
    pub owner: Option<(EntityUuid, EntityId)>,
    pub position: DVec3,
    pub velocity: DVec3,
    pub critical: bool,
}

type Mobs<'w, 's> = Query<'w, 's, (&'static MotionId, &'static MotionState, &'static Mob)>;
type Players<'w, 's> =
    Query<'w, 's, (&'static Owner, &'static Transform, PlayerBody), With<Player>>;

pub(crate) fn aim_ranged_attacks(
    mut attacks: MessageReader<RangedAttack>,
    mobs: Mobs,
    players: Players,
    births: EntityBirths,
    mut launches: MessageWriter<ProjectileLaunch>,
) {
    for attack in attacks.read() {
        let Some((_, state, mob)) = mobs.iter().find(|(id, ..)| id.0 == attack.attacker) else {
            continue;
        };
        let Some(target) = aimed(&mobs, &players, attack.target) else {
            continue;
        };
        let shooter = Shooter {
            entity_type: mob.kind.entity_type,
            feet: state.0.body.position,
            eye_height: mob.kind.eye_height,
        };
        let stream = KeyedStream::new(&state.0.key, births.tick());
        let Some(shot) = mob_shot(&shooter, &target, &stream) else {
            continue;
        };
        launches.write(ProjectileLaunch {
            kind: shot.kind,
            owner: Some((state.0.key, attack.attacker)),
            position: shot.position,
            velocity: shot.velocity,
            critical: false,
        });
    }
}

fn aimed(mobs: &Mobs<'_, '_>, players: &Players<'_, '_>, target: EntityId) -> Option<Aimed> {
    if let Some((_, state, mob)) = mobs.iter().find(|(id, ..)| id.0 == target) {
        return Some(Aimed {
            feet: state.0.body.position,
            height: mob.kind.height,
            eye_height: mob.kind.eye_height,
        });
    }
    let (_, transform, body) = players.iter().find(|(owner, ..)| {
        EntityId(u64::from(player_network_id(owner.0).cast_unsigned())) == target
    })?;
    let dimensions = body.dimensions();
    Some(Aimed {
        feet: transform.translation,
        height: dimensions.height,
        eye_height: dimensions.eye_height,
    })
}

pub(crate) fn launch_projectiles(
    mut launches: MessageReader<ProjectileLaunch>,
    births: EntityBirths,
    mut sequences: Local<BirthSequences>,
    mut commands: Commands,
) {
    for launch in launches.read() {
        let source = match launch.owner {
            Some((key, _)) => BirthSource::Entity(key),
            None => BirthSource::Block(block_at(launch.position)),
        };
        let key = births.born(&source, sequences.next(births.tick(), &source));
        let body = Body {
            position: launch.position,
            velocity: launch.velocity,
            ..Body::default()
        };
        let mut state = EntityState::new(key, body, None);
        state.phase = key.phase_offset();
        let mut projectile = Projectile::new(launch.kind, launch.owner, launch.velocity.length());
        projectile.critical = launch.critical;
        let data = launch
            .owner
            .map_or(0, |(_, id)| (id.0 as u32).cast_signed());
        commands.spawn((
            MotionState(state),
            MotionKind(launch.kind.row()),
            Appearance {
                kind: launch.kind.kind(),
                look: SpawnLook::Custom {
                    data,
                    entries: Vec::new(),
                },
            },
            projectile,
        ));
    }
}
