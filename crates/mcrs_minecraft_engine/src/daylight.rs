use mcrs_minecraft_dimension_environment::dimension_type::DimensionTypeFile;
use mcrs_minecraft_engine_world::world::dimension::DimensionTypeId;
use mcrs_minecraft_environment::WorldClocks;
use mcrs_minecraft_registries::CurrentRegistries;
use mcrs_minecraft_resource::Id;

#[must_use]
pub fn day_ticks(
    registries: Option<&CurrentRegistries>,
    clocks: Option<&WorldClocks>,
    dimension_type: Option<&DimensionTypeId>,
    fallback: i64,
) -> i64 {
    let clock = registries
        .and_then(|registries| registries.registry::<DimensionTypeFile>())
        .and_then(|types| {
            let id = Id::checked(dimension_type?.0.number(), types.len())?;
            types.get(id)?.default_clock
        });
    clock
        .and_then(|clock| clocks?.get(clock))
        .map_or(fallback, |time| time.total_ticks)
}
