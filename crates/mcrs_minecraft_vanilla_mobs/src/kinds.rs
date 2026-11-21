use mcrs_minecraft_brain::Dimensions;
use mcrs_minecraft_brain::KindData;
use mcrs_minecraft_brain::KindFlags;
use mcrs_minecraft_brain::KindTable;
use mcrs_minecraft_brain::MobKind;
use mcrs_minecraft_entity::entity_type_keys::EntityType;

#[must_use]
pub const fn kind(entity_type: EntityType) -> MobKind {
    MobKind(entity_type.id().number())
}

#[must_use]
pub fn kind_table() -> KindTable {
    KindTable::new(
        KINDS
            .iter()
            .map(|(entity_type, data)| (kind(*entity_type), *data)),
    )
}

#[rustfmt::skip]
pub const KINDS: &[(EntityType, KindData)] = &[
    (EntityType::Allay, KindData::new(Dimensions::new(0.35, 0.6, 0.36), KindFlags::NONE)),
    (EntityType::Armadillo, KindData::new(Dimensions::new(0.7, 0.65, 0.26), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Axolotl, KindData::new(Dimensions::new(0.75, 0.42, 0.2751), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Bat, KindData::new(Dimensions::new(0.5, 0.9, 0.45), KindFlags::NONE)),
    (EntityType::Bee, KindData::new(Dimensions::new(0.55, 0.5, 0.3), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Blaze, KindData::new(Dimensions::new(0.6, 1.8, 1.53), KindFlags::FIRE_IMMUNE)),
    (EntityType::Bogged, KindData::new(Dimensions::new(0.6, 1.99, 1.74), KindFlags::NONE)),
    (EntityType::Breeze, KindData::new(Dimensions::new(0.6, 1.77, 1.3452), KindFlags::NONE)),
    (EntityType::Camel, KindData::new(Dimensions::new(1.7, 2.375, 2.275), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME))),
    (EntityType::CamelHusk, KindData::new(Dimensions::new(1.7, 2.375, 2.275), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Cat, KindData::new(Dimensions::new(0.6, 0.7, 0.35), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME))),
    (EntityType::CaveSpider, KindData::new(Dimensions::new(0.7, 0.5, 0.45), KindFlags::NONE)),
    (EntityType::Chicken, KindData::new(Dimensions::new(0.4, 0.7, 0.644), KindFlags::AGEABLE.union(KindFlags::ANIMAL)).with_baby(Dimensions::new(0.3, 0.4, 0.28125))),
    (EntityType::Cod, KindData::new(Dimensions::new(0.5, 0.3, 0.195), KindFlags::NONE)),
    (EntityType::CopperGolem, KindData::new(Dimensions::new(0.49, 0.98, 0.8125), KindFlags::NONE)),
    (EntityType::Cow, KindData::new(Dimensions::new(0.9, 1.4, 1.3), KindFlags::AGEABLE.union(KindFlags::ANIMAL)).with_baby(Dimensions::new(0.45, 0.7, 0.69))),
    (EntityType::Creaking, KindData::new(Dimensions::new(0.9, 2.7, 2.3), KindFlags::NONE)),
    (EntityType::Creeper, KindData::new(Dimensions::new(0.6, 1.7, 1.445), KindFlags::NONE)),
    (EntityType::Dolphin, KindData::new(Dimensions::new(0.9, 0.6, 0.3), KindFlags::AGEABLE)),
    (EntityType::Donkey, KindData::new(Dimensions::new(1.3964844, 1.5, 1.425), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME))),
    (EntityType::Drowned, KindData::new(Dimensions::new(0.6, 1.95, 1.74), KindFlags::AGEABLE)),
    (EntityType::ElderGuardian, KindData::new(Dimensions::new(1.9975, 1.9975, 0.99875), KindFlags::NONE)),
    (EntityType::EnderDragon, KindData::new(Dimensions::new(16.0, 8.0, 6.8), KindFlags::FIRE_IMMUNE)),
    (EntityType::Enderman, KindData::new(Dimensions::new(0.6, 2.9, 2.55), KindFlags::NONE)),
    (EntityType::Endermite, KindData::new(Dimensions::new(0.4, 0.3, 0.13), KindFlags::NONE)),
    (EntityType::Evoker, KindData::new(Dimensions::new(0.6, 1.95, 1.6575), KindFlags::NONE)),
    (EntityType::Fox, KindData::new(Dimensions::new(0.6, 0.7, 0.4), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Frog, KindData::new(Dimensions::new(0.5, 0.5, 0.425), KindFlags::AGEABLE.union(KindFlags::ANIMAL)).with_head(10.0, 40.0, 5.0)),
    (EntityType::Frostbite, KindData::new(Dimensions::new(0.6, 1.95, 1.74), KindFlags::NONE)),
    (EntityType::Ghast, KindData::new(Dimensions::new(4.0, 4.0, 2.6), KindFlags::FIRE_IMMUNE.union(KindFlags::UNTARGETABLE))),
    (EntityType::Giant, KindData::new(Dimensions::new(3.6, 12.0, 10.44), KindFlags::NONE)),
    (EntityType::GlowSquid, KindData::new(Dimensions::new(0.8, 0.8, 0.4), KindFlags::AGEABLE)),
    (EntityType::Goat, KindData::new(Dimensions::new(0.9, 1.3, 1.105), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Guardian, KindData::new(Dimensions::new(0.85, 0.85, 0.425), KindFlags::NONE)),
    (EntityType::HappyGhast, KindData::new(Dimensions::new(4.0, 4.0, 2.6), KindFlags::NONE)),
    (EntityType::Hoglin, KindData::new(Dimensions::new(1.3964844, 1.4, 1.19), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Horse, KindData::new(Dimensions::new(1.3964844, 1.6, 1.52), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME))),
    (EntityType::Husk, KindData::new(Dimensions::new(0.6, 1.95, 1.74), KindFlags::AGEABLE)),
    (EntityType::Illusioner, KindData::new(Dimensions::new(0.6, 1.95, 1.6575), KindFlags::NONE)),
    (EntityType::IronGolem, KindData::new(Dimensions::new(1.4, 2.7, 2.295), KindFlags::NONE)),
    (EntityType::Llama, KindData::new(Dimensions::new(0.9, 1.87, 1.7765), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME)).with_baby(Dimensions::new(0.45, 0.935, 0.88825))),
    (EntityType::MagmaCube, KindData::new(Dimensions::new(0.52, 0.52, 0.325), KindFlags::FIRE_IMMUNE)),
    (EntityType::Mooshroom, KindData::new(Dimensions::new(0.9, 1.4, 1.3), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Mule, KindData::new(Dimensions::new(1.3964844, 1.6, 1.52), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME))),
    (EntityType::Nautilus, KindData::new(Dimensions::new(0.875, 0.95, 0.2751), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Ocelot, KindData::new(Dimensions::new(0.6, 0.7, 0.595), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Panda, KindData::new(Dimensions::new(1.3, 1.25, 1.0625), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Parched, KindData::new(Dimensions::new(0.6, 1.99, 1.74), KindFlags::NONE)),
    (EntityType::Parrot, KindData::new(Dimensions::new(0.5, 0.9, 0.54), KindFlags::CAN_BE_TAME)),
    (EntityType::Phantom, KindData::new(Dimensions::new(0.9, 0.5, 0.175), KindFlags::NONE)),
    (EntityType::Pig, KindData::new(Dimensions::new(0.9, 0.9, 0.765), KindFlags::AGEABLE.union(KindFlags::ANIMAL)).with_baby(Dimensions::new(0.45, 0.45, 0.40625))),
    (EntityType::Piglin, KindData::new(Dimensions::new(0.6, 1.95, 1.79), KindFlags::AGEABLE)),
    (EntityType::PiglinBrute, KindData::new(Dimensions::new(0.6, 1.95, 1.79), KindFlags::NONE)),
    (EntityType::Pillager, KindData::new(Dimensions::new(0.6, 1.95, 1.6575), KindFlags::NONE)),
    (EntityType::PolarBear, KindData::new(Dimensions::new(1.4, 1.4, 1.19), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Pufferfish, KindData::new(Dimensions::new(0.7, 0.7, 0.455), KindFlags::NONE)),
    (EntityType::Rabbit, KindData::new(Dimensions::new(0.49, 0.6, 0.59), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Ravager, KindData::new(Dimensions::new(1.95, 2.2, 1.8700001), KindFlags::NONE)),
    (EntityType::Salmon, KindData::new(Dimensions::new(0.7, 0.4, 0.26), KindFlags::NONE)),
    (EntityType::Sheep, KindData::new(Dimensions::new(0.9, 1.3, 1.235), KindFlags::AGEABLE.union(KindFlags::ANIMAL)).with_baby(Dimensions::new(0.45, 0.65, 0.65625))),
    (EntityType::Shulker, KindData::new(Dimensions::new(1.0, 1.0, 0.5), KindFlags::FIRE_IMMUNE)),
    (EntityType::Silverfish, KindData::new(Dimensions::new(0.4, 0.3, 0.13), KindFlags::NONE)),
    (EntityType::Skeleton, KindData::new(Dimensions::new(0.6, 1.99, 1.74), KindFlags::NONE)),
    (EntityType::SkeletonHorse, KindData::new(Dimensions::new(1.3964844, 1.6, 1.52), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME))),
    (EntityType::Slime, KindData::new(Dimensions::new(0.52, 0.52, 0.325), KindFlags::NONE)),
    (EntityType::Sniffer, KindData::new(Dimensions::new(1.9, 1.75, 1.05), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::SnowGolem, KindData::new(Dimensions::new(0.7, 1.9, 1.7), KindFlags::NONE)),
    (EntityType::Spider, KindData::new(Dimensions::new(1.4, 0.9, 0.65), KindFlags::NONE)),
    (EntityType::Squid, KindData::new(Dimensions::new(0.8, 0.8, 0.4), KindFlags::AGEABLE)),
    (EntityType::Stray, KindData::new(Dimensions::new(0.6, 1.99, 1.74), KindFlags::NONE)),
    (EntityType::Strider, KindData::new(Dimensions::new(0.9, 1.7, 1.445), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::FIRE_IMMUNE))),
    (EntityType::SulfurCube, KindData::new(Dimensions::new(0.49, 0.49, 0.175), KindFlags::NONE)),
    (EntityType::Tadpole, KindData::new(Dimensions::new(0.4, 0.3, 0.19500001), KindFlags::AGEABLE)),
    (EntityType::TraderLlama, KindData::new(Dimensions::new(0.9, 1.87, 1.7765), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME)).with_baby(Dimensions::new(0.45, 0.935, 0.88825))),
    (EntityType::TropicalFish, KindData::new(Dimensions::new(0.5, 0.4, 0.26), KindFlags::NONE)),
    (EntityType::Turtle, KindData::new(Dimensions::new(1.2, 0.4, 0.34), KindFlags::AGEABLE.union(KindFlags::ANIMAL))),
    (EntityType::Vex, KindData::new(Dimensions::new(0.4, 0.8, 0.51875), KindFlags::FIRE_IMMUNE)),
    (EntityType::Villager, KindData::new(Dimensions::new(0.6, 1.95, 1.62), KindFlags::AGEABLE)),
    (EntityType::Vindicator, KindData::new(Dimensions::new(0.6, 1.95, 1.6575), KindFlags::NONE)),
    (EntityType::WanderingTrader, KindData::new(Dimensions::new(0.6, 1.95, 1.62), KindFlags::AGEABLE)),
    (EntityType::Warden, KindData::new(Dimensions::new(0.9, 2.9, 2.4650002), KindFlags::FIRE_IMMUNE)),
    (EntityType::Witch, KindData::new(Dimensions::new(0.6, 1.95, 1.62), KindFlags::NONE)),
    (EntityType::Wither, KindData::new(Dimensions::new(0.9, 3.5, 2.9750001), KindFlags::FIRE_IMMUNE)),
    (EntityType::WitherSkeleton, KindData::new(Dimensions::new(0.7, 2.4, 2.1), KindFlags::FIRE_IMMUNE)),
    (EntityType::Wolf, KindData::new(Dimensions::new(0.6, 0.85, 0.68), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME).union(KindFlags::PANIC_ENVIRONMENTAL_ONLY)).with_baby(Dimensions::new(0.3, 0.425, 0.34375))),
    (EntityType::Zoglin, KindData::new(Dimensions::new(1.3964844, 1.4, 1.19), KindFlags::AGEABLE.union(KindFlags::FIRE_IMMUNE))),
    (EntityType::Zombie, KindData::new(Dimensions::new(0.6, 1.95, 1.74), KindFlags::AGEABLE)),
    (EntityType::ZombieHorse, KindData::new(Dimensions::new(1.3964844, 1.6, 1.52), KindFlags::AGEABLE.union(KindFlags::ANIMAL).union(KindFlags::CAN_BE_TAME))),
    (EntityType::ZombieNautilus, KindData::new(Dimensions::new(0.875, 0.95, 0.2751), KindFlags::NONE)),
    (EntityType::ZombieVillager, KindData::new(Dimensions::new(0.6, 1.95, 1.74), KindFlags::AGEABLE)),
    (EntityType::ZombifiedPiglin, KindData::new(Dimensions::new(0.6, 1.95, 1.79), KindFlags::AGEABLE.union(KindFlags::FIRE_IMMUNE))),
];
