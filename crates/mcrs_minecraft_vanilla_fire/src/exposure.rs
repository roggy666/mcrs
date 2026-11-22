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
use mcrs_minecraft_engine::PlayerBody;
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
use mcrs_minecraft_physics::Aabb;
use mcrs_minecraft_physics::CollisionTable;
use mcrs_minecraft_physics::Contact;
use mcrs_minecraft_physics::block_at;
use mcrs_minecraft_registries::CurrentRegistries;
use mcrs_minecraft_volume::BlockVolume;
use mcrs_voxel::BlockPos;
use mcrs_voxel::ColumnPos;
use mcrs_voxel::LocalPos;
use mcrs_voxel::SectionPos;

use crate::sun::SunSensitive;
use crate::work::FireWork;

const FULL_SKY_LIGHT: u8 = 15;
const CLIP: f64 = 1.0e-5_f32 as f64;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct Daylight {
    pub bright: bool,
    pub brightness: f64,
    pub raining: bool,
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

fn box_contact(view: &DimensionView<'_>, table: &CollisionTable, body: &Aabb) -> Contact {
    let min = body.min + DVec3::splat(CLIP);
    let max = body.max - DVec3::splat(CLIP);
    let (low, high) = (block_at(min), block_at(max));
    let mut contact = Contact::empty();
    for z in low.z..=high.z {
        for y in low.y..=high.y {
            for x in low.x..=high.x {
                if let Ok(state) = view.block(BlockPos::new(x, y, z)) {
                    contact |= Contact::of_block(table.state(state).flags);
                }
            }
        }
    }
    contact
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

type SunMobs<'w, 's> =
    Query<'w, 's, (&'static MotionState, &'static mut Exposure), (With<Mob>, With<SunSensitive>)>;

type BurningMobs<'w, 's> = Query<
    'w,
    's,
    (&'static MotionState, &'static mut Exposure),
    (With<Mob>, Without<SunSensitive>),
>;

type Players<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, PlayerBody, &'static mut Exposure),
    (With<Player>, Without<Mob>),
>;

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
    work: Res<FireWork>,
    mut daylight: ResMut<Daylight>,
    mut sun_mobs: SunMobs,
    mut burning: BurningMobs,
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
    let bright = has_sky && is_bright_outside(darken);
    daylight.set_if_neq(Daylight {
        bright,
        brightness: sun_brightness(darken),
        raining,
    });
    let sky = Sky {
        sections,
        columns,
        lights: &lights,
        heights: &heights,
    };
    if bright || raining {
        sun_mobs.par_iter_mut().for_each(|(state, mut exposure)| {
            let pos = block_at(state.0.body.position);
            let open = sky.open_above(pos);
            let in_water = state.0.contact.in_water();
            let in_rain = raining && open;
            let in_sun =
                bright && open && !in_water && !in_rain && sky.sky_light(pos) == FULL_SKY_LIGHT;
            exposure.set_if_neq(Exposure {
                in_sun,
                in_water,
                in_rain,
                ..Exposure::default()
            });
        });
    } else {
        sun_mobs.par_iter_mut().for_each(|(state, mut exposure)| {
            exposure.set_if_neq(Exposure {
                in_water: state.0.contact.in_water(),
                ..Exposure::default()
            });
        });
    }
    for entity in work.entities() {
        if let Ok((state, mut exposure)) = burning.get_mut(*entity) {
            let pos = block_at(state.0.body.position);
            exposure.set_if_neq(Exposure {
                in_water: state.0.contact.in_water(),
                in_rain: raining && sky.open_above(pos),
                ..Exposure::default()
            });
        }
    }
    for (transform, body, mut exposure) in &mut players {
        let dimensions = body.dimensions();
        let feet = transform.translation;
        let bounds = Aabb::of_body(feet, dimensions.width, dimensions.height);
        let contact = box_contact(&view, table, &bounds);
        let pos = block_at(feet);
        exposure.set_if_neq(Exposure {
            in_sun: false,
            in_water: contact.in_water(),
            in_rain: raining && sky.open_above(pos),
            in_lava: contact.in_lava(),
            in_fire: contact.contains(Contact::FIRE),
            in_soul_fire: contact.contains(Contact::SOUL_FIRE),
        });
    }
}
