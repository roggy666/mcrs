use bevy_ecs::entity::Entity;
use bevy_ecs::message::Messages;
use bevy_ecs::prelude::With;
use mcrs_minecraft_chunk::{LightStorage, SkyLight};
use mcrs_minecraft_command_engine::CommandPlugin;
use mcrs_minecraft_command_vanilla::{VanillaCommands, VanillaCommandsPlugin};
use mcrs_minecraft_damage::{Exposure, Fire, Vitals};
use mcrs_minecraft_dimension_environment::dimension_type::DimensionTypeFile;
use mcrs_minecraft_engine_world::block_update::BlockSetRequest;
use mcrs_minecraft_engine_world::world::dimension::Dimension;
use mcrs_minecraft_engine_world::world::dimension::DimensionTypeId;
use mcrs_minecraft_engine_world::world::storage::column::ColumnIndex;
use mcrs_minecraft_engine_world::world::storage::section::SectionIndex;
use mcrs_minecraft_environment::{ClockTime, WorldClocks};
use mcrs_minecraft_heightmap::MotionHeightmap;
use mcrs_minecraft_protocol::Bounded;
use mcrs_minecraft_protocol::uuid::Uuid;
use mcrs_minecraft_protocol_game::serverbound::ServerboundChatCommand;
use mcrs_minecraft_registries::CurrentRegistries;
use mcrs_minecraft_resource::Id;
use mcrs_minecraft_server::{
    VanillaEntityAttributesPlugin, VanillaEntityPhysicsPlugin, VanillaFirePlugin,
    VanillaMobBrainPlugin,
};
use mcrs_minecraft_simulation::BlockUpdateFlags;
use mcrs_voxel::BlockPos;
use mcrs_voxel::ColumnPos;
use mcrs_voxel::SectionPos;

use crate::commands::load_stone_column;
use crate::host_app::{self, PlayServer};
use crate::mob_replication::mob_entities;

const OPERATOR: Uuid = Uuid::from_u128(7);
const ZOMBIE_HEALTH: f32 = 20.0;
const SUN_TICKS: usize = 400;

fn start() -> (PlayServer, Entity) {
    let mut app = host_app::make_host_app();
    app.add_plugins(CommandPlugin::<VanillaCommands>::new());
    app.add_plugins(VanillaCommandsPlugin);
    app.add_plugins(VanillaEntityPhysicsPlugin);
    app.add_plugins(VanillaEntityAttributesPlugin);
    app.add_plugins(VanillaMobBrainPlugin);
    app.add_plugins(VanillaFirePlugin);
    let mut server = PlayServer::start(app, &[(OPERATOR, 2)], 0);
    load_stone_column(&mut server);
    light_the_sky(&mut server);
    let (operator, _) = server.join(OPERATOR, "op");
    (server, operator)
}

fn light_the_sky(server: &mut PlayServer) {
    let world = server.world();
    let dimension = world
        .query_filtered::<Entity, With<Dimension>>()
        .single(world)
        .unwrap();
    let sections: Vec<(i32, Entity)> = (0..16)
        .filter_map(|chunk_y| {
            world
                .get::<SectionIndex>(dimension)?
                .get(SectionPos::new(0, chunk_y, 0))
                .map(|entity| (chunk_y, entity))
        })
        .collect();
    for (chunk_y, section) in sections {
        let level = if chunk_y >= 4 { 15 } else { 0 };
        world
            .entity_mut(section)
            .insert(SkyLight(LightStorage::from_field(
                &[level; SectionPos::VOLUME],
            )));
    }
}

fn run(server: &mut PlayServer, operator: Entity, line: &str) {
    server.send(
        operator,
        &ServerboundChatCommand {
            command: Bounded(line),
        },
    );
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

fn dimension(server: &mut PlayServer) -> Entity {
    let world = server.world();
    world
        .query_filtered::<Entity, With<Dimension>>()
        .single(world)
        .unwrap()
}

fn roof_over(server: &mut PlayServer, pos: BlockPos) {
    set_stone(server, pos);
    let dimension = dimension(server);
    let world = server.world();
    let column = world
        .get::<ColumnIndex>(dimension)
        .and_then(|index| index.get(&ColumnPos::from(pos)))
        .map(|slot| slot.entity)
        .unwrap();
    let mut heights = world.get_mut::<MotionHeightmap>(column).unwrap();
    let (x, z) = ((pos.x & 15) as usize, (pos.z & 15) as usize);
    heights.0.set(x, z, pos.y + 1);
}

fn set_day_time(server: &mut PlayServer, ticks: i64) {
    let dimension = dimension(server);
    let world = server.world();
    let dimension_type = world.get::<DimensionTypeId>(dimension).copied();
    let clock = world
        .resource::<CurrentRegistries>()
        .registry::<DimensionTypeFile>()
        .and_then(|types| {
            let id = Id::checked(dimension_type?.0.number(), types.len())?;
            types.get(id)?.default_clock
        })
        .expect("the dimension type has a default clock");
    let mut clocks = WorldClocks::default();
    clocks.insert(
        clock,
        ClockTime {
            total_ticks: ticks,
            ..ClockTime::default()
        },
    );
    world.insert_resource(clocks);
}

fn summon(server: &mut PlayServer, operator: Entity, kind: &str) -> Entity {
    run(
        server,
        operator,
        &format!("summon minecraft:{kind} 4 64 4 {{NoAI:1b}}"),
    );
    server.ticks(operator, 5);
    let mobs = mob_entities(server);
    assert_eq!(mobs.len(), 1, "one {kind} stands on the floor");
    mobs[0].entity
}

struct Seen {
    burned: bool,
    sunlit: bool,
}

fn watch(server: &mut PlayServer, operator: Entity, mob: Entity, ticks: usize) -> Seen {
    let mut seen = Seen {
        burned: false,
        sunlit: false,
    };
    for _ in 0..ticks {
        server.ticks(operator, 1);
        let world = server.world();
        seen.burned |= world.get::<Fire>(mob).is_some_and(Fire::burning);
        seen.sunlit |= world
            .get::<Exposure>(mob)
            .is_some_and(|exposure| exposure.in_sun);
    }
    seen
}

fn health(server: &mut PlayServer, mob: Entity) -> f32 {
    server.world().get::<Vitals>(mob).unwrap().health
}

#[test]
fn mon_sun_004_a_zombie_in_the_sun_ignites_and_burns() {
    let (mut server, operator) = start();
    let zombie = summon(&mut server, operator, "zombie");
    let seen = watch(&mut server, operator, zombie, SUN_TICKS);
    assert!(
        seen.sunlit,
        "a zombie on the stone column by day is in the sun"
    );
    assert!(seen.burned, "the sun ignites the zombie");
    assert!(
        health(&mut server, zombie) < ZOMBIE_HEALTH,
        "the on_fire hits lower its health"
    );
}

#[test]
fn mon_sun_004_a_zombie_under_a_roof_stays_cold() {
    let (mut server, operator) = start();
    roof_over(&mut server, BlockPos::new(4, 68, 4));
    server.ticks(operator, 5);
    let zombie = summon(&mut server, operator, "zombie");
    let seen = watch(&mut server, operator, zombie, SUN_TICKS);
    assert!(!seen.sunlit, "a roof keeps the zombie out of the sun");
    assert!(!seen.burned);
    assert_eq!(health(&mut server, zombie), ZOMBIE_HEALTH);
}

#[test]
fn mon_sun_004_a_husk_does_not_burn_in_the_sun() {
    let (mut server, operator) = start();
    let husk = summon(&mut server, operator, "husk");
    let seen = watch(&mut server, operator, husk, SUN_TICKS);
    assert!(!seen.sunlit, "a husk is not sun sensitive");
    assert!(!seen.burned);
    assert_eq!(health(&mut server, husk), ZOMBIE_HEALTH);
}

#[test]
fn mon_sun_004_the_sun_does_not_burn_at_midnight() {
    let (mut server, operator) = start();
    set_day_time(&mut server, 18_000);
    server.ticks(operator, 2);
    let zombie = summon(&mut server, operator, "zombie");
    let seen = watch(&mut server, operator, zombie, SUN_TICKS);
    assert!(!seen.sunlit, "the sun is down at midnight");
    assert!(!seen.burned);
    assert_eq!(health(&mut server, zombie), ZOMBIE_HEALTH);
}

#[test]
fn env_fir_002_an_ignited_zombie_takes_eight_hits_and_burns_out_under_a_roof() {
    let (mut server, operator) = start();
    roof_over(&mut server, BlockPos::new(4, 68, 4));
    server.ticks(operator, 5);
    let zombie = summon(&mut server, operator, "zombie");
    server.world().get_mut::<Fire>(zombie).unwrap().ignite(160);
    server.ticks(operator, 165);
    let fire = server.world().get::<Fire>(zombie).copied().unwrap();
    assert!(
        !fire.burning(),
        "the fire burns out after 160 ticks: {fire:?}"
    );
    assert_eq!(
        health(&mut server, zombie),
        ZOMBIE_HEALTH - 8.0,
        "eight on_fire hits of 1.0 land"
    );
}
