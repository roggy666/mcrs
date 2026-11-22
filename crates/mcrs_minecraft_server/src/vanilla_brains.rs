use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_vanilla_mobs::VanillaBrain;

pub(crate) fn vanilla_brain(kind: &'static EntityTypeData) -> Option<VanillaBrain> {
    mcrs_minecraft_vanilla_mobs::vanilla_brain(kind)
}
