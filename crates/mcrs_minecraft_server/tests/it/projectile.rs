use bevy_ecs::entity::Entity;
use bevy_ecs::message::Messages;
use bevy_ecs::prelude::With;
use bevy_math::DVec3;
use mcrs_minecraft_command_engine::CommandPlugin;
use mcrs_minecraft_command_vanilla::{VanillaCommands, VanillaCommandsPlugin};
use mcrs_minecraft_damage::Vitals;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_world::block_update::BlockSetRequest;
use mcrs_minecraft_engine_world::world::dimension::Dimension;
use mcrs_minecraft_game_mode::{GameMode, PlayerGameMode};
use mcrs_minecraft_protocol::Bounded;
use mcrs_minecraft_protocol::uuid::Uuid;
use mcrs_minecraft_protocol_game::serverbound::ServerboundChatCommand;
use mcrs_minecraft_server::{
    VanillaEntityPhysicsPlugin, VanillaMobBrainPlugin, VanillaProjectilePlugin,
};
use mcrs_minecraft_simulation::BlockUpdateFlags;
use mcrs_minecraft_vanilla_projectile::{Projectile, ProjectileKind, ProjectileLaunch};
use mcrs_voxel::BlockPos;

use crate::commands::load_stone_column;
use crate::host_app::{self, PlayServer};
use crate::mob_replication::mob_entities;

const OPERATOR: Uuid = Uuid::from_u128(1);

fn start() -> (PlayServer, Entity) {
    let mut app = host_app::make_host_app();
    app.add_plugins(CommandPlugin::<VanillaCommands>::new());
    app.add_plugins(VanillaCommandsPlugin);
    app.add_plugins(VanillaEntityPhysicsPlugin);
    app.add_plugins(VanillaMobBrainPlugin);
    app.add_plugins(VanillaProjectilePlugin);
    let mut server = PlayServer::start(app, &[(OPERATOR, 2)], 0);
    load_stone_column(&mut server);
    let (operator, _) = server.join(OPERATOR, "op");
    (server, operator)
}

fn run(server: &mut PlayServer, operator: Entity, line: &str) {
    server.send(
        operator,
        &ServerboundChatCommand {
            command: Bounded(line),
        },
    );
}

fn launch(server: &mut PlayServer, kind: ProjectileKind, position: DVec3, velocity: DVec3) {
    server
        .world()
        .resource_mut::<Messages<ProjectileLaunch>>()
        .write(ProjectileLaunch {
            kind,
            owner: None,
            position,
            velocity,
            critical: false,
        });
}

fn projectiles(server: &mut PlayServer) -> Vec<(Projectile, MotionState)> {
    let world = server.world();
    world
        .query::<(&Projectile, &MotionState)>()
        .iter(world)
        .map(|(projectile, state)| (*projectile, *state))
        .collect()
}

fn set_stone(server: &mut PlayServer, pos: BlockPos) {
    let stone = crate::support::standalone_corpus()
        .0
        .default_state("minecraft:stone")
        .unwrap();
    let world = server.world();
    let dimension = world
        .query_filtered::<Entity, With<Dimension>>()
        .single(world)
        .unwrap();
    world
        .resource_mut::<Messages<BlockSetRequest>>()
        .write(BlockSetRequest {
            dimension,
            pos,
            new_state: stone,
            flags: BlockUpdateFlags::all(),
            recursion_left: 1,
        });
}

#[test]
fn prj_ent_002_an_arrow_hits_a_pig_for_the_amount_of_its_speed() {
    let (mut server, operator) = start();
    run(
        &mut server,
        operator,
        "summon minecraft:pig 6 64 4 {NoAI:1b}",
    );
    server.ticks(operator, 10);
    let pig = mob_entities(&mut server);
    assert_eq!(pig.len(), 1, "one pig stands on the floor");
    let pig = pig[0].entity;
    let before = server.world().get::<Vitals>(pig).unwrap().health;

    launch(
        &mut server,
        ProjectileKind::Arrow,
        DVec3::new(0.5, 64.5, 4.5),
        DVec3::new(1.5, 0.0, 0.0),
    );
    server.ticks(operator, 12);

    let after = server.world().get::<Vitals>(pig).unwrap().health;
    assert_eq!(
        before - after,
        3.0,
        "an arrow at the speed 1.5 with the base damage 2 hits for ceil(3.0)"
    );
    assert!(
        projectiles(&mut server).is_empty(),
        "an accepted hit removes the arrow"
    );
}

#[test]
fn prj_blk_001_an_arrow_stops_in_a_wall_and_stays() {
    let (mut server, operator) = start();
    for y in 64..=66 {
        set_stone(&mut server, BlockPos::new(8, y, 4));
    }
    server.ticks(operator, 2);
    launch(
        &mut server,
        ProjectileKind::Arrow,
        DVec3::new(2.5, 65.0, 4.5),
        DVec3::new(1.5, 0.0, 0.0),
    );
    server.ticks(operator, 10);

    let stuck = projectiles(&mut server);
    assert_eq!(stuck.len(), 1, "the arrow stays in the wall");
    let (arrow, state) = stuck[0];
    assert!(
        arrow.in_ground,
        "the arrow is in the in-ground state: {arrow:?} {:?}",
        state.0.body
    );
    assert_eq!(state.0.body.velocity, DVec3::ZERO);
    let x = state.0.body.position.x;
    assert!(
        (7.9..8.0).contains(&x),
        "the arrow stops a back-off before the wall face: {x}"
    );

    server.ticks(operator, 40);
    let later = projectiles(&mut server);
    assert_eq!(later.len(), 1, "the arrow waits for its despawn counter");
    assert_eq!(
        later[0].1.0.body.position.x, x,
        "a stuck arrow does not move"
    );
}

#[test]
fn prj_mob_001_a_skeleton_shoots_an_arrow_at_its_target() {
    let (mut server, operator) = start();
    let player = server.player(operator);
    server.world().get_mut::<PlayerGameMode>(player).unwrap().0 = GameMode::Survival;
    run(&mut server, operator, "summon minecraft:skeleton 4 64 4");
    run(
        &mut server,
        operator,
        "summon minecraft:pig 12 64 4 {NoAI:1b}",
    );
    let mut shot = None;
    for _ in 0..200 {
        server.ticks(operator, 1);
        if let Some(first) = projectiles(&mut server).first() {
            shot = Some(*first);
            break;
        }
    }
    let (arrow, state) = shot.expect("the skeleton shoots within 200 ticks");
    assert_eq!(arrow.kind, ProjectileKind::Arrow);
    assert!(arrow.owner.is_some(), "the arrow names its shooter");
    assert!(
        (arrow.speed - 1.6).abs() < 0.2,
        "a mob arrow leaves at about the speed 1.6: {}",
        arrow.speed
    );
    assert!(
        state.0.body.position.x > 3.0,
        "the arrow starts at the skeleton: {:?}",
        state.0.body.position
    );
}
