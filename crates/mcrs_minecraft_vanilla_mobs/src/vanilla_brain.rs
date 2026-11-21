use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::TableBrain;

use crate::CHICKEN;
use crate::COW;
use crate::LLAMA;
use crate::PIG;
use crate::SHEEP;
use crate::WOLF;

pub type VanillaBrain = TableBrain;

#[must_use]
pub fn table_of(entity_type: EntityType) -> Option<&'static GoalTable> {
    match entity_type {
        EntityType::Pig => Some(&PIG),
        EntityType::Cow => Some(&COW),
        EntityType::Sheep => Some(&SHEEP),
        EntityType::Chicken => Some(&CHICKEN),
        EntityType::Wolf => Some(&WOLF),
        EntityType::Llama => Some(&LLAMA),
        _ => None,
    }
}

#[must_use]
pub fn vanilla_brain(kind: &'static EntityTypeData) -> Option<VanillaBrain> {
    table_of(kind.entity_type).map(TableBrain::new)
}

const _: () = assert!(
    size_of::<VanillaBrain>() <= 256,
    "each mob of a tile pays the size of the brain in the brain column"
);
