use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::Has;
use bevy_ecs::query::With;
use bevy_ecs::query::Without;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use glam::DVec3;
use mcrs_minecraft_chunk::SkyLight;
use mcrs_minecraft_damage::Exposure;
use mcrs_minecraft_engine::coordinator::clock::GameTime;
use mcrs_minecraft_engine::daylight::day_ticks;
use mcrs_minecraft_engine::server_edit::Weather;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_physics::Mob;
use mcrs_minecraft_engine_physics::PhysicsTables;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::world::dimension::Dimension;
use mcrs_minecraft_engine_world::world::dimension::DimensionTypeId;
use mcrs_minecraft_engine_world::world::dimension::HasSkyLight;
use mcrs_minecraft_engine_world::world::dimension::HasWeather;
use mcrs_minecraft_engine_world::world::storage::column::ColumnIndex;
use mcrs_minecraft_engine_world::world::storage::section::SectionIndex;
use mcrs_minecraft_engine_world::world::storage::view::DimensionView;
use mcrs_minecraft_engine_world::world::storage::view::WorldView;
use mcrs_minecraft_environment::WorldClocks;
use mcrs_minecraft_environment::daylight::is_bright_outside;
use mcrs_minecraft_environment::daylight::sky_darken;
use mcrs_minecraft_environment::daylight::sun_brightness;
use mcrs_minecraft_heightmap::MotionHeightmap;
use mcrs_minecraft_physics::BlockFlags;
use mcrs_minecraft_physics::CollisionTable;
use mcrs_minecraft_physics::block_at;
use mcrs_minecraft_registries::CurrentRegistries;
use mcrs_minecraft_volume::BlockVolume;
use mcrs_voxel::BlockPos;
use mcrs_voxel::ColumnPos;
use mcrs_voxel::LocalPos;
use mcrs_voxel::SectionPos;

use crate::sun::SunSensitive;

const FULL_SKY_LIGHT: u8 = 15;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct Daylight {
    pub bright: bool,
    pub brightness: f64,
    pub tick: u64,
}

struct Sky<'a> {
    sections: &'a SectionIndex,
    columns: &'a ColumnIndex,
    lights: &'a Query<'a, 'a, &'static SkyLight>,
    heights: &'a Query<'a, 'a, &'static MotionHeightmap>,
}

impl Sky<'_> {
    fn sky_light(&self, pos: BlockPos) -> u8 {
        let Some(entity) = self.sections.get(SectionPos::from(pos)) else {
            return 0;
        };
        let local = LocalPos::from(pos);
        self.lights.get(entity).map_or(0, |light| {
            light.0.get(
                usize::from(local.x()),
                usize::from(local.y()),
                usize::from(local.z()),
            )
        })
    }

    fn open_above(&self, pos: BlockPos) -> bool {
        let Some(slot) = self.columns.get(&ColumnPos::from(pos)) else {
            return false;
        };
        let local = LocalPos::from(pos);
        self.heights.get(slot.entity).is_ok_and(|heights| {
            pos.y
                >= heights
                    .0
                    .get(usize::from(local.x()), usize::from(local.z()))
        })
    }
}

fn in_fluid(view: &DimensionView<'_>, table: &CollisionTable, position: DVec3) -> bool {
    view.block(block_at(position))
        .is_ok_and(|state| table.state(state).flags.contains(BlockFlags::FLUID))
}

type Dimensions<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SectionIndex,
        &'static ColumnIndex,
        Option<&'static DimensionTypeId>,
        Has<HasSkyLight>,
        Has<HasWeather>,
    ),
    With<Dimension>,
>;

type Mobs<'w, 's> = Query<
    'w,
    's,
    (
        &'static MotionState,
        Has<SunSensitive>,
        &'static mut Exposure,
    ),
    With<Mob>,
>;

type Players<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static mut Exposure), (With<Player>, Without<Mob>)>;

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each query and resource is its own parameter"
)]
pub(crate) fn sense_exposure(
    dims: Dimensions,
    world: WorldView,
    tables: Res<PhysicsTables>,
    lights: Query<&'static SkyLight>,
    heights: Query<&'static MotionHeightmap>,
    time: Res<GameTime>,
    registries: Option<Res<CurrentRegistries>>,
    clocks: Option<Res<WorldClocks>>,
    weather: Option<Res<Weather>>,
    mut daylight: ResMut<Daylight>,
    mut mobs: Mobs,
    mut players: Players,
) {
    let Ok((dimension, sections, columns, dimension_type, has_sky, has_weather)) = dims.single()
    else {
        return;
    };
    let Some(view) = world.get(dimension) else {
        return;
    };
    let table = tables.collision();
    let raining = has_weather && weather.as_deref().is_some_and(|weather| weather.raining);
    let thundering = has_weather && weather.as_deref().is_some_and(|weather| weather.thundering);
    let ticks = day_ticks(
        registries.as_deref(),
        clocks.as_deref(),
        dimension_type,
        time.0,
    );
    let darken = sky_darken(
        ticks,
        if raining { 1.0 } else { 0.0 },
        if thundering { 1.0 } else { 0.0 },
    );
    let tick = daylight.tick.wrapping_add(1);
    daylight.set_if_neq(Daylight {
        bright: has_sky && is_bright_outside(darken),
        brightness: sun_brightness(darken),
        tick,
    });
    let sky = Sky {
        sections,
        columns,
        lights: &lights,
        heights: &heights,
    };
    let bright = daylight.bright;
    for (state, sun_sensitive, mut exposure) in &mut mobs {
        let position = state.0.body.position;
        let pos = block_at(position);
        let in_water = in_fluid(&view, table, position);
        let open = (raining || (sun_sensitive && bright)) && sky.open_above(pos);
        let in_rain = raining && open;
        let in_sun = sun_sensitive
            && bright
            && open
            && !in_water
            && !in_rain
            && sky.sky_light(pos) == FULL_SKY_LIGHT;
        exposure.set_if_neq(Exposure {
            in_sun,
            in_water,
            in_rain,
        });
    }
    for (transform, mut exposure) in &mut players {
        let position = transform.translation;
        let pos = block_at(position);
        let in_water = in_fluid(&view, table, position);
        let in_rain = raining && sky.open_above(pos);
        exposure.set_if_neq(Exposure {
            in_sun: false,
            in_water,
            in_rain,
        });
    }
}
