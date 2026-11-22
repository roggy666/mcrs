use bevy_ecs::entity::Entity;
use bevy_ecs::message::Messages;
use bevy_ecs::prelude::With;
use bevy_math::DVec3;
use mcrs_minecraft_command_engine::CommandPlugin;
use mcrs_minecraft_command_vanilla::{VanillaCommands, VanillaCommandsPlugin};
use mcrs_minecraft_core::{Direction, Look};
use mcrs_minecraft_damage::Vitals;
use mcrs_minecraft_engine::bus::PacketPayload;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_item::{HolderCommandsExt, Slots};
use mcrs_minecraft_engine_registries::{install_item_table, test_item_table};
use mcrs_minecraft_engine_world::block_update::BlockSetRequest;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::session::Owner;
use mcrs_minecraft_engine_world::world::dimension::Dimension;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_game_mode::{GameMode, PlayerGameMode};
use mcrs_minecraft_item_key::item_keys::Item;
use mcrs_minecraft_item_stack::{Count, ItemStack, Kind};
use mcrs_minecraft_protocol::Bounded;
use mcrs_minecraft_protocol::VarInt;
use mcrs_minecraft_protocol::uuid::Uuid;
use mcrs_minecraft_protocol_game::Hand;
use mcrs_minecraft_protocol_game::player::PlayerAction;
use mcrs_minecraft_protocol_game::serverbound::ServerboundChatCommand;
use mcrs_minecraft_protocol_game::serverbound::{
    ServerboundAcceptTeleportation, ServerboundMovePlayerPos, ServerboundPlayerAction,
    ServerboundUseItem,
};
use mcrs_minecraft_server::VanillaItemEntityPlugin;
use mcrs_minecraft_server::{
    VanillaEntityAttributesPlugin, VanillaEntityPhysicsPlugin, VanillaMobBrainPlugin,
    VanillaProjectilePlugin,
};
use mcrs_minecraft_simulation::BlockUpdateFlags;
use mcrs_minecraft_vanilla_projectile::{Drawing, Pickup, player_id};
use mcrs_minecraft_vanilla_projectile::{Projectile, ProjectileKind, ProjectileLaunch};
use mcrs_voxel::BlockPos;

use crate::commands::load_stone_column;
use crate::host_app::{self, PlayServer, sent_to};
use crate::mob_replication::mob_entities;

const OPERATOR: Uuid = Uuid::from_u128(1);

fn start() -> (PlayServer, Entity) {
    let mut app = host_app::make_host_app();
    app.add_plugins(CommandPlugin::<VanillaCommands>::new());
    app.add_plugins(VanillaCommandsPlugin);
    app.add_plugins(VanillaEntityPhysicsPlugin);
    app.add_plugins(VanillaEntityAttributesPlugin);
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
            pickup: Pickup::Disallowed,
            sound: None,
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
    set_block(server, pos, "minecraft:stone");
}

fn set_block(server: &mut PlayServer, pos: BlockPos, name: &str) {
    let stone = crate::support::standalone_corpus()
        .0
        .default_state(name)
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

fn start_with_items() -> (PlayServer, Entity, Entity) {
    let mut app = host_app::make_host_app();
    install_item_table(app.world_mut(), test_item_table()).unwrap();
    app.add_plugins(CommandPlugin::<VanillaCommands>::new());
    app.add_plugins(VanillaCommandsPlugin);
    app.add_plugins(VanillaEntityPhysicsPlugin);
    app.add_plugins(VanillaEntityAttributesPlugin);
    app.add_plugins(VanillaItemEntityPlugin);
    app.add_plugins(VanillaMobBrainPlugin);
    app.add_plugins(VanillaProjectilePlugin);
    let mut server = PlayServer::start(app, &[(OPERATOR, 2)], 0);
    load_stone_column(&mut server);
    let (operator, packets) = server.join(OPERATOR, "op");
    accept_teleports(&mut server, operator, &packets);
    let player = server.player(operator);
    server.world().get_mut::<PlayerGameMode>(player).unwrap().0 = GameMode::Survival;
    (server, operator, player)
}

fn accept_teleports(server: &mut PlayServer, operator: Entity, packets: &[PacketPayload]) {
    for packet in packets {
        let PacketPayload::PlayerPosition(teleport) = packet else {
            continue;
        };
        server.queue(
            operator,
            &ServerboundAcceptTeleportation {
                teleport_id: teleport.teleport_id,
                position: teleport.position.into(),
                look: Look::default(),
            },
        );
    }
    server.ticks(operator, 2);
}

fn give(server: &mut PlayServer, player: Entity, item: Item, count: u8) {
    let world = server.world();
    world.commands().entity(player).give(ItemStack::new(
        Kind(item.id().number()),
        Count::new(count).unwrap(),
    ));
    world.flush();
}

fn slot_count(server: &mut PlayServer, player: Entity, slot: usize) -> Option<u8> {
    let world = server.world();
    let stack = world.get::<Slots>(player)?.get(slot)?;
    world.get::<Count>(stack).map(|count| count.get())
}

#[test]
fn prj_lch_001_a_full_draw_shoots_a_critical_arrow_and_spends_one() {
    let (mut server, operator, player) = start_with_items();
    give(&mut server, player, Item::Bow, 1);
    give(&mut server, player, Item::Arrow, 64);
    server.ticks(operator, 2);
    assert_eq!(slot_count(&mut server, player, 1), Some(64));

    server.queue(
        operator,
        &ServerboundUseItem {
            hand: Hand::Main,
            sequence: VarInt(1),
            look: Look {
                yaw: 0.0,
                pitch: 0.0,
            },
        },
    );
    server.ticks(operator, 20);
    {
        let world = server.world();
        let drawing = world.get::<Drawing>(player).copied();
        let kinds: Vec<Option<u16>> = world
            .get::<Slots>(player)
            .map(|slots| {
                slots
                    .iter()
                    .take(3)
                    .map(|stack| {
                        stack
                            .and_then(|stack| world.get::<Kind>(stack))
                            .map(|kind| kind.0)
                    })
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            drawing.is_some(),
            "the use starts a draw: slots {kinds:?}, bow {}",
            Item::Bow.id().number()
        );
    }
    server.queue(
        operator,
        &ServerboundPlayerAction {
            action: PlayerAction::ReleaseUseItem,
            pos: BlockPos::new(0, 0, 0),
            direction: Direction::Down,
            sequence: VarInt(2),
        },
    );
    server.ticks(operator, 2);

    let shot = projectiles(&mut server);
    let still_drawing = server.world().get::<Drawing>(player).copied();
    let power = mcrs_minecraft_vanilla_projectile::bow_power(20);
    assert_eq!(
        shot.len(),
        1,
        "the release launches one arrow: drawing {still_drawing:?}, power {power}"
    );
    let (arrow, state) = shot[0];
    assert_eq!(arrow.kind, ProjectileKind::Arrow);
    assert!(arrow.critical, "a full draw makes a critical arrow");
    assert_eq!(arrow.pickup, Pickup::Allowed);
    assert!(
        (arrow.speed - 3.0).abs() < 0.1,
        "a full draw leaves at the speed 3: {}",
        arrow.speed
    );
    assert!(
        state.0.body.velocity.z > 2.5,
        "the arrow flies where the player looks: {:?}",
        state.0.body.velocity
    );
    assert_eq!(
        slot_count(&mut server, player, 1),
        Some(63),
        "the shot takes one arrow"
    );
}

#[test]
fn prj_ent_001_an_arrow_hits_a_player_and_the_owner_gets_its_health() {
    let (mut server, operator, player) = start_with_items();
    let session = server.session_of(operator).unwrap();
    let before = server.world().get::<Vitals>(player).unwrap().health;
    let feet = server.world().get::<Transform>(player).unwrap().translation;
    launch(
        &mut server,
        ProjectileKind::Arrow,
        feet + DVec3::new(-3.0, 0.9, 0.0),
        DVec3::new(1.5, 0.0, 0.0),
    );
    let packets = server.ticks_all(12);
    let after = server.world().get::<Vitals>(player).unwrap().health;
    assert_eq!(before - after, 3.0, "the player takes the arrow");
    let healths: Vec<f32> = sent_to(session, packets)
        .into_iter()
        .filter_map(|payload| match payload {
            PacketPayload::SetHealth(health) => Some(health.health),
            _ => None,
        })
        .collect();
    assert_eq!(healths, vec![after], "the owner gets the new health once");
    assert!(projectiles(&mut server).is_empty());
}

#[test]
fn prj_ent_007_a_full_arrow_hit_pushes_the_pig_away() {
    let (mut server, operator) = start();
    run(
        &mut server,
        operator,
        "summon minecraft:pig 6 64 4 {NoAI:1b}",
    );
    server.ticks(operator, 10);
    let pig = mob_entities(&mut server)[0].entity;
    let before = server
        .world()
        .get::<MotionState>(pig)
        .unwrap()
        .0
        .body
        .position;
    launch(
        &mut server,
        ProjectileKind::Arrow,
        DVec3::new(0.5, 64.5, 4.5),
        DVec3::new(1.5, 0.0, 0.0),
    );
    server.ticks(operator, 12);
    let after = server
        .world()
        .get::<MotionState>(pig)
        .unwrap()
        .0
        .body
        .position;
    assert!(
        after.x - before.x > 0.3,
        "the pig moves away from the arrow: {before:?} -> {after:?}"
    );
}

#[test]
fn prj_pck_001_a_player_picks_up_an_arrow_stuck_in_a_block() {
    let (mut server, operator, player) = start_with_items();
    let feet = server.world().get::<Transform>(player).unwrap().translation;
    let wall = BlockPos::new(feet.x.floor() as i32 + 1, 64, feet.z.floor() as i32);
    set_stone(&mut server, wall);
    set_stone(&mut server, BlockPos::new(wall.x, 65, wall.z));
    server.ticks(operator, 2);
    let key = *server.world().get::<EntityUuid>(player).unwrap();
    let owner = player_id(server.world().get::<Owner>(player).unwrap());
    server
        .world()
        .resource_mut::<Messages<ProjectileLaunch>>()
        .write(ProjectileLaunch {
            kind: ProjectileKind::Arrow,
            owner: Some((key, owner)),
            position: feet + DVec3::new(-0.2, 0.6, 0.0),
            velocity: DVec3::new(1.5, 0.0, 0.0),
            critical: false,
            pickup: Pickup::Allowed,
            sound: None,
        });
    server.ticks(operator, 4);
    let stuck = projectiles(&mut server);
    assert_eq!(stuck.len(), 1, "the arrow sticks in the wall by the player");
    assert!(stuck[0].0.in_ground);

    server.ticks(operator, 10);
    assert!(
        projectiles(&mut server).is_empty(),
        "the player picks the arrow up once its shake ends"
    );
    assert_eq!(
        slot_count(&mut server, player, 0),
        Some(1),
        "one arrow item enters the inventory"
    );
}

#[test]
fn prj_lch_004_an_arrow_inherits_the_walk_of_its_shooter() {
    const STEP: f64 = 0.3;
    let (mut server, operator, player) = start_with_items();
    give(&mut server, player, Item::Bow, 1);
    give(&mut server, player, Item::Arrow, 64);
    server.ticks(operator, 2);
    let start = server.world().get::<Transform>(player).unwrap().translation;

    server.queue(
        operator,
        &ServerboundUseItem {
            hand: Hand::Main,
            sequence: VarInt(1),
            look: Look {
                yaw: 0.0,
                pitch: 0.0,
            },
        },
    );
    for step in 1..=20 {
        server.queue(
            operator,
            &ServerboundMovePlayerPos {
                position: (start + DVec3::new(STEP * f64::from(step), 0.0, 0.0)).into(),
                flags: mcrs_minecraft_protocol_game::MoveFlags::new().with_on_ground(true),
            },
        );
        server.ticks(operator, 1);
    }
    let walked = server.world().get::<Transform>(player).unwrap().translation;
    assert!(
        (walked.x - start.x - STEP * 20.0).abs() < 1e-6,
        "the moves apply: {start} -> {walked}"
    );
    server.queue(
        operator,
        &ServerboundPlayerAction {
            action: PlayerAction::ReleaseUseItem,
            pos: BlockPos::new(0, 0, 0),
            direction: Direction::Down,
            sequence: VarInt(2),
        },
    );
    server.ticks(operator, 1);

    let shot = projectiles(&mut server);
    assert_eq!(shot.len(), 1, "the release launches one arrow");
    let velocity = shot[0].1.0.body.velocity;
    assert!(
        (velocity.x - STEP).abs() < 0.05,
        "the arrow carries the walk of the shooter along x: {velocity}"
    );
    assert!(
        velocity.y.abs() < 0.05,
        "a shooter on the ground gives no vertical motion: {velocity}"
    );
}

#[test]
fn path_a_zombie_walks_around_a_fence_to_its_target() {
    let (mut server, operator) = start();
    let player = server.player(operator);
    server.world().get_mut::<PlayerGameMode>(player).unwrap().0 = GameMode::Survival;
    let feet = server.world().get::<Transform>(player).unwrap().translation;
    let wall_x = BlockPos::from(feet).x + 4;
    for z in -5..=5 {
        set_block(
            &mut server,
            BlockPos::new(wall_x, 64, BlockPos::from(feet).z + z),
            "minecraft:oak_fence",
        );
    }
    server.ticks(operator, 3);
    run(
        &mut server,
        operator,
        &format!(
            "summon minecraft:zombie {} 64 {}",
            wall_x + 2,
            BlockPos::from(feet).z
        ),
    );
    server.ticks(operator, 5);
    let zombie = mob_entities(&mut server)[0].entity;
    let mut best = f64::INFINITY;
    for _ in 0..400 {
        server.ticks(operator, 1);
        let position = server
            .world()
            .get::<MotionState>(zombie)
            .unwrap()
            .0
            .body
            .position;
        best = best.min(position.distance(feet));
        if best < 2.5 {
            break;
        }
    }
    assert!(
        best < 2.5,
        "the zombie walks around the fence to the player: closest {best}"
    );
}
