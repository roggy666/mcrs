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
    if kind.entity_type == EntityType::Parrot {
        row.descent_damping = Some(PARROT_DESCENT_DAMPING);
    }
    if matches!(
        kind.entity_type,
        EntityType::Parrot | EntityType::Bee | EntityType::Allay | EntityType::Bat
    ) {
        row.fall_check = false;
    }
    row
}

const PARROT_DESCENT_DAMPING: f64 = 0.6;
