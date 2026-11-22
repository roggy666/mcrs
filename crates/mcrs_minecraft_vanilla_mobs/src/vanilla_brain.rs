use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::TableBrain;

use crate::BEE_TABLE;
use crate::BLAZE;
use crate::CAT_TABLE;
use crate::CHICKEN;
use crate::COW;
use crate::CREEPER;
use crate::ENDERMAN;
use crate::EVOKER;
use crate::FOX_TABLE;
use crate::HORSE_TABLE;
use crate::ILLUSIONER;
use crate::IRON_GOLEM_TABLE;
use crate::LLAMA;
use crate::OCELOT_TABLE;
use crate::PANDA_TABLE;
use crate::PARROT_TABLE;
use crate::PIG;
use crate::PILLAGER;
use crate::POLAR_BEAR_TABLE;
use crate::RABBIT_TABLE;
use crate::RAVAGER;
use crate::SHEEP;
use crate::SILVERFISH;
use crate::SKELETON;
use crate::SNOW_GOLEM;
use crate::SPIDER;
use crate::STILL;
use crate::STRIDER_TABLE;
use crate::TRADER_LLAMA_TABLE;
use crate::TURTLE_TABLE;
use crate::UNDEAD_HORSE_TABLE;
use crate::VINDICATOR;
use crate::WANDERING_TRADER_TABLE;
use crate::WITCH;
use crate::WITHER_SKELETON;
use crate::WOLF;
use crate::ZOMBIE;
use crate::ZOMBIFIED_PIGLIN_TABLE;

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
        EntityType::TraderLlama => Some(&TRADER_LLAMA_TABLE),
        EntityType::Mooshroom => Some(&COW),
        EntityType::Zombie
        | EntityType::Husk
        | EntityType::Drowned
        | EntityType::ZombieVillager => Some(&ZOMBIE),
        EntityType::ZombifiedPiglin => Some(&ZOMBIFIED_PIGLIN_TABLE),
        EntityType::Skeleton | EntityType::Stray | EntityType::Bogged | EntityType::Parched => {
            Some(&SKELETON)
        }
        EntityType::WitherSkeleton => Some(&WITHER_SKELETON),
        EntityType::Creeper => Some(&CREEPER),
        EntityType::Spider | EntityType::CaveSpider => Some(&SPIDER),
        EntityType::Enderman => Some(&ENDERMAN),
        EntityType::Silverfish | EntityType::Endermite => Some(&SILVERFISH),
        EntityType::Witch => Some(&WITCH),
        EntityType::Blaze => Some(&BLAZE),
        EntityType::IronGolem => Some(&IRON_GOLEM_TABLE),
        EntityType::SnowGolem => Some(&SNOW_GOLEM),
        EntityType::Pillager => Some(&PILLAGER),
        EntityType::Vindicator => Some(&VINDICATOR),
        EntityType::Evoker => Some(&EVOKER),
        EntityType::Illusioner => Some(&ILLUSIONER),
        EntityType::Ravager => Some(&RAVAGER),
        EntityType::Cat => Some(&CAT_TABLE),
        EntityType::Ocelot => Some(&OCELOT_TABLE),
        EntityType::Rabbit => Some(&RABBIT_TABLE),
        EntityType::PolarBear => Some(&POLAR_BEAR_TABLE),
        EntityType::Panda => Some(&PANDA_TABLE),
        EntityType::Horse | EntityType::Donkey | EntityType::Mule => Some(&HORSE_TABLE),
        EntityType::SkeletonHorse | EntityType::ZombieHorse => Some(&UNDEAD_HORSE_TABLE),
        EntityType::WanderingTrader => Some(&WANDERING_TRADER_TABLE),
        EntityType::Fox => Some(&FOX_TABLE),
        EntityType::Turtle => Some(&TURTLE_TABLE),
        EntityType::Strider => Some(&STRIDER_TABLE),
        EntityType::Parrot => Some(&PARROT_TABLE),
        EntityType::Bee => Some(&BEE_TABLE),
        EntityType::Slime
        | EntityType::MagmaCube
        | EntityType::Giant
        | EntityType::Vex
        | EntityType::Bat
        | EntityType::Squid
        | EntityType::GlowSquid
        | EntityType::Cod
        | EntityType::Salmon
        | EntityType::TropicalFish
        | EntityType::Pufferfish
        | EntityType::Dolphin => Some(&STILL),
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
