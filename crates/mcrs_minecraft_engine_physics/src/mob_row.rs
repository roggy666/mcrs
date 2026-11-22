use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_physics::Category;
use mcrs_minecraft_physics::MotionRow;
use mcrs_minecraft_pose::Dimensions;

#[must_use]
pub fn mob_row(kind: &EntityTypeData) -> MotionRow {
    let category = match kind.entity_type {
        EntityType::Ghast | EntityType::HappyGhast | EntityType::Phantom | EntityType::Allay => {
            Category::Flier
        }
        EntityType::Bee | EntityType::Parrot => Category::Omnidirectional,
        _ => Category::Walker,
    };
    let mut row = MotionRow::living(
        category,
        Dimensions::scalable(kind.width, kind.height).with_eye_height(kind.eye_height),
    );
    if category == Category::Flier {
        row.gravity = 0.0;
    }
    row
}
