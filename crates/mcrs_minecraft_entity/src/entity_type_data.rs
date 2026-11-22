use crate::client_motion::ClientMotion;
use crate::entity_type_keys::EntityType;
use crate::life::Life;
use crate::mob_data::MobData;
use crate::precise_sync::PreciseSync;
use crate::variant_indices::VariantIndices;
use mcrs_minecraft_entity_attribute::AttributeBases;

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct EntityTypeData {
    pub entity_type: EntityType,
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub tracking_range: u32,
    pub update_interval: u32,
    pub tracks_velocity: bool,
    pub client_motion: ClientMotion,
    pub fire_immune: bool,
    pub baby_scale: Option<f32>,
    pub walk_target_factor: f32,
    pub life: Life,
    pub variant_indices: VariantIndices,
}

const MOB_DEFAULTS: EntityTypeData = EntityTypeData {
    entity_type: EntityType::Pig,
    width: 0.0,
    height: 0.0,
    eye_height: 0.0,
    tracking_range: 10,
    update_interval: EntityTypeData::DEFAULT_UPDATE_INTERVAL,
    tracks_velocity: true,
    client_motion: ClientMotion::Replayed,
    fire_immune: false,
    baby_scale: None,
    walk_target_factor: 4.0,
    life: Life::Mob(MobData::DEFAULT),
    variant_indices: VariantIndices::NONE,
};

const ANIMAL: Life = Life::Mob(MobData {
    ageable: true,
    ..MobData::DEFAULT
});

impl EntityTypeData {
    pub const DEFAULT_UPDATE_INTERVAL: u32 = 3;

    pub const PIG: Self = Self {
        entity_type: EntityType::Pig,
        width: 0.9,
        height: 0.9,
        eye_height: 0.765,
        baby_scale: Some(0.5),
        life: ANIMAL,
        variant_indices: VariantIndices::PIG,
        ..MOB_DEFAULTS
    };

    pub const COW: Self = Self {
        entity_type: EntityType::Cow,
        width: 0.9,
        height: 1.4,
        eye_height: 1.3,
        baby_scale: Some(0.5),
        life: ANIMAL,
        variant_indices: VariantIndices::COW,
        ..MOB_DEFAULTS
    };

    pub const SHEEP: Self = Self {
        entity_type: EntityType::Sheep,
        width: 0.9,
        height: 1.3,
        eye_height: 1.235,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const CHICKEN: Self = Self {
        entity_type: EntityType::Chicken,
        width: 0.4,
        height: 0.7,
        eye_height: 0.644,
        life: ANIMAL,
        variant_indices: VariantIndices::CHICKEN,
        ..MOB_DEFAULTS
    };

    pub const ZOMBIFIED_PIGLIN: Self = Self {
        entity_type: EntityType::ZombifiedPiglin,
        width: 0.6,
        height: 1.95,
        eye_height: 1.79,
        tracking_range: 8,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const WOLF: Self = Self {
        entity_type: EntityType::Wolf,
        width: 0.6,
        height: 0.85,
        eye_height: 0.68,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const LLAMA: Self = Self {
        entity_type: EntityType::Llama,
        width: 0.9,
        height: 1.87,
        eye_height: 1.7765,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const FROG: Self = Self {
        entity_type: EntityType::Frog,
        width: 0.5,
        height: 0.5,
        eye_height: 0.425,
        walk_target_factor: 25.0,
        life: Life::Mob(MobData {
            head_yaw_limit: 5.0,
            ageable: true,
            ..MobData::DEFAULT
        }),
        ..MOB_DEFAULTS
    };

    pub const ALLAY: Self = Self {
        entity_type: EntityType::Allay,
        width: 0.35,
        height: 0.6,
        eye_height: 0.36,
        tracking_range: 8,
        update_interval: 2,
        ..MOB_DEFAULTS
    };

    pub const ARMADILLO: Self = Self {
        entity_type: EntityType::Armadillo,
        width: 0.7,
        height: 0.65,
        eye_height: 0.26,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const AXOLOTL: Self = Self {
        entity_type: EntityType::Axolotl,
        width: 0.75,
        height: 0.42,
        eye_height: 0.2751,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const BAT: Self = Self {
        entity_type: EntityType::Bat,
        width: 0.5,
        height: 0.9,
        eye_height: 0.45,
        tracking_range: 5,
        ..MOB_DEFAULTS
    };

    pub const BEE: Self = Self {
        entity_type: EntityType::Bee,
        width: 0.55,
        height: 0.5,
        eye_height: 0.3,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const BLAZE: Self = Self {
        entity_type: EntityType::Blaze,
        width: 0.6,
        height: 1.8,
        eye_height: 1.53,
        tracking_range: 8,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const BOGGED: Self = Self {
        entity_type: EntityType::Bogged,
        width: 0.6,
        height: 1.99,
        eye_height: 1.74,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const BREEZE: Self = Self {
        entity_type: EntityType::Breeze,
        width: 0.6,
        height: 1.77,
        eye_height: 1.3452,
        ..MOB_DEFAULTS
    };

    pub const CAMEL: Self = Self {
        entity_type: EntityType::Camel,
        width: 1.7,
        height: 2.375,
        eye_height: 2.275,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const CAMEL_HUSK: Self = Self {
        entity_type: EntityType::CamelHusk,
        width: 1.7,
        height: 2.375,
        eye_height: 2.275,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const CAT: Self = Self {
        entity_type: EntityType::Cat,
        width: 0.6,
        height: 0.7,
        eye_height: 0.35,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const CAVE_SPIDER: Self = Self {
        entity_type: EntityType::CaveSpider,
        width: 0.7,
        height: 0.5,
        eye_height: 0.45,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const COD: Self = Self {
        entity_type: EntityType::Cod,
        width: 0.5,
        height: 0.3,
        eye_height: 0.195,
        tracking_range: 4,
        ..MOB_DEFAULTS
    };

    pub const COPPER_GOLEM: Self = Self {
        entity_type: EntityType::CopperGolem,
        width: 0.49,
        height: 0.98,
        eye_height: 0.8125,
        ..MOB_DEFAULTS
    };

    pub const CREAKING: Self = Self {
        entity_type: EntityType::Creaking,
        width: 0.9,
        height: 2.7,
        eye_height: 2.3,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const CREEPER: Self = Self {
        entity_type: EntityType::Creeper,
        width: 0.6,
        height: 1.7,
        eye_height: 1.445,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const DOLPHIN: Self = Self {
        entity_type: EntityType::Dolphin,
        width: 0.9,
        height: 0.6,
        eye_height: 0.3,
        tracking_range: 5,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const DONKEY: Self = Self {
        entity_type: EntityType::Donkey,
        width: 1.3964844,
        height: 1.5,
        eye_height: 1.425,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const DROWNED: Self = Self {
        entity_type: EntityType::Drowned,
        width: 0.6,
        height: 1.95,
        eye_height: 1.74,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const ELDER_GUARDIAN: Self = Self {
        entity_type: EntityType::ElderGuardian,
        width: 1.9975,
        height: 1.9975,
        eye_height: 0.99875,
        ..MOB_DEFAULTS
    };

    pub const ENDER_DRAGON: Self = Self {
        entity_type: EntityType::EnderDragon,
        width: 16.0,
        height: 8.0,
        eye_height: 6.8,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const ENDERMAN: Self = Self {
        entity_type: EntityType::Enderman,
        width: 0.6,
        height: 2.9,
        eye_height: 2.55,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const ENDERMITE: Self = Self {
        entity_type: EntityType::Endermite,
        width: 0.4,
        height: 0.3,
        eye_height: 0.13,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const EVOKER: Self = Self {
        entity_type: EntityType::Evoker,
        width: 0.6,
        height: 1.95,
        eye_height: 1.6575,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const FOX: Self = Self {
        entity_type: EntityType::Fox,
        width: 0.6,
        height: 0.7,
        eye_height: 0.4,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const FROSTBITE: Self = Self {
        entity_type: EntityType::Frostbite,
        width: 0.6,
        height: 1.95,
        eye_height: 1.74,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const GHAST: Self = Self {
        entity_type: EntityType::Ghast,
        width: 4.0,
        height: 4.0,
        eye_height: 2.6,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const GIANT: Self = Self {
        entity_type: EntityType::Giant,
        width: 3.6,
        height: 12.0,
        eye_height: 10.44,
        ..MOB_DEFAULTS
    };

    pub const GLOW_SQUID: Self = Self {
        entity_type: EntityType::GlowSquid,
        width: 0.8,
        height: 0.8,
        eye_height: 0.4,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const GOAT: Self = Self {
        entity_type: EntityType::Goat,
        width: 0.9,
        height: 1.3,
        eye_height: 1.105,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const GUARDIAN: Self = Self {
        entity_type: EntityType::Guardian,
        width: 0.85,
        height: 0.85,
        eye_height: 0.425,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const HAPPY_GHAST: Self = Self {
        entity_type: EntityType::HappyGhast,
        width: 4.0,
        height: 4.0,
        eye_height: 2.6,
        ..MOB_DEFAULTS
    };

    pub const HOGLIN: Self = Self {
        entity_type: EntityType::Hoglin,
        width: 1.3964844,
        height: 1.4,
        eye_height: 1.19,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const HORSE: Self = Self {
        entity_type: EntityType::Horse,
        width: 1.3964844,
        height: 1.6,
        eye_height: 1.52,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const HUSK: Self = Self {
        entity_type: EntityType::Husk,
        width: 0.6,
        height: 1.95,
        eye_height: 1.74,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const ILLUSIONER: Self = Self {
        entity_type: EntityType::Illusioner,
        width: 0.6,
        height: 1.95,
        eye_height: 1.6575,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const IRON_GOLEM: Self = Self {
        entity_type: EntityType::IronGolem,
        width: 1.4,
        height: 2.7,
        eye_height: 2.295,
        ..MOB_DEFAULTS
    };

    pub const MAGMA_CUBE: Self = Self {
        entity_type: EntityType::MagmaCube,
        width: 0.52,
        height: 0.52,
        eye_height: 0.325,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const MOOSHROOM: Self = Self {
        entity_type: EntityType::Mooshroom,
        width: 0.9,
        height: 1.4,
        eye_height: 1.3,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const MULE: Self = Self {
        entity_type: EntityType::Mule,
        width: 1.3964844,
        height: 1.6,
        eye_height: 1.52,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const NAUTILUS: Self = Self {
        entity_type: EntityType::Nautilus,
        width: 0.875,
        height: 0.95,
        eye_height: 0.2751,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const OCELOT: Self = Self {
        entity_type: EntityType::Ocelot,
        width: 0.6,
        height: 0.7,
        eye_height: 0.595,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const PANDA: Self = Self {
        entity_type: EntityType::Panda,
        width: 1.3,
        height: 1.25,
        eye_height: 1.0625,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const PARCHED: Self = Self {
        entity_type: EntityType::Parched,
        width: 0.6,
        height: 1.99,
        eye_height: 1.74,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const PARROT: Self = Self {
        entity_type: EntityType::Parrot,
        width: 0.5,
        height: 0.9,
        eye_height: 0.54,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const PHANTOM: Self = Self {
        entity_type: EntityType::Phantom,
        width: 0.9,
        height: 0.5,
        eye_height: 0.175,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const PIGLIN: Self = Self {
        entity_type: EntityType::Piglin,
        width: 0.6,
        height: 1.95,
        eye_height: 1.79,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const PIGLIN_BRUTE: Self = Self {
        entity_type: EntityType::PiglinBrute,
        width: 0.6,
        height: 1.95,
        eye_height: 1.79,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const PILLAGER: Self = Self {
        entity_type: EntityType::Pillager,
        width: 0.6,
        height: 1.95,
        eye_height: 1.6575,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const POLAR_BEAR: Self = Self {
        entity_type: EntityType::PolarBear,
        width: 1.4,
        height: 1.4,
        eye_height: 1.19,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const PUFFERFISH: Self = Self {
        entity_type: EntityType::Pufferfish,
        width: 0.7,
        height: 0.7,
        eye_height: 0.455,
        tracking_range: 4,
        ..MOB_DEFAULTS
    };

    pub const RABBIT: Self = Self {
        entity_type: EntityType::Rabbit,
        width: 0.49,
        height: 0.6,
        eye_height: 0.59,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const RAVAGER: Self = Self {
        entity_type: EntityType::Ravager,
        width: 1.95,
        height: 2.2,
        eye_height: 1.8700001,
        ..MOB_DEFAULTS
    };

    pub const SALMON: Self = Self {
        entity_type: EntityType::Salmon,
        width: 0.7,
        height: 0.4,
        eye_height: 0.26,
        tracking_range: 4,
        ..MOB_DEFAULTS
    };

    pub const SHULKER: Self = Self {
        entity_type: EntityType::Shulker,
        width: 1.0,
        height: 1.0,
        eye_height: 0.5,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const SILVERFISH: Self = Self {
        entity_type: EntityType::Silverfish,
        width: 0.4,
        height: 0.3,
        eye_height: 0.13,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const SKELETON: Self = Self {
        entity_type: EntityType::Skeleton,
        width: 0.6,
        height: 1.99,
        eye_height: 1.74,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const SKELETON_HORSE: Self = Self {
        entity_type: EntityType::SkeletonHorse,
        width: 1.3964844,
        height: 1.6,
        eye_height: 1.52,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const SLIME: Self = Self {
        entity_type: EntityType::Slime,
        width: 0.52,
        height: 0.52,
        eye_height: 0.325,
        ..MOB_DEFAULTS
    };

    pub const SNIFFER: Self = Self {
        entity_type: EntityType::Sniffer,
        width: 1.9,
        height: 1.75,
        eye_height: 1.05,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const SNOW_GOLEM: Self = Self {
        entity_type: EntityType::SnowGolem,
        width: 0.7,
        height: 1.9,
        eye_height: 1.7,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const SPIDER: Self = Self {
        entity_type: EntityType::Spider,
        width: 1.4,
        height: 0.9,
        eye_height: 0.65,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const SQUID: Self = Self {
        entity_type: EntityType::Squid,
        width: 0.8,
        height: 0.8,
        eye_height: 0.4,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const STRAY: Self = Self {
        entity_type: EntityType::Stray,
        width: 0.6,
        height: 1.99,
        eye_height: 1.74,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const STRIDER: Self = Self {
        entity_type: EntityType::Strider,
        width: 0.9,
        height: 1.7,
        eye_height: 1.445,
        fire_immune: true,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const SULFUR_CUBE: Self = Self {
        entity_type: EntityType::SulfurCube,
        width: 0.49,
        height: 0.49,
        eye_height: 0.175,
        ..MOB_DEFAULTS
    };

    pub const TADPOLE: Self = Self {
        entity_type: EntityType::Tadpole,
        width: 0.4,
        height: 0.3,
        eye_height: 0.19500001,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const TRADER_LLAMA: Self = Self {
        entity_type: EntityType::TraderLlama,
        width: 0.9,
        height: 1.87,
        eye_height: 1.7765,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const TROPICAL_FISH: Self = Self {
        entity_type: EntityType::TropicalFish,
        width: 0.5,
        height: 0.4,
        eye_height: 0.26,
        tracking_range: 4,
        ..MOB_DEFAULTS
    };

    pub const TURTLE: Self = Self {
        entity_type: EntityType::Turtle,
        width: 1.2,
        height: 0.4,
        eye_height: 0.34,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const VEX: Self = Self {
        entity_type: EntityType::Vex,
        width: 0.4,
        height: 0.8,
        eye_height: 0.51875,
        tracking_range: 8,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const VILLAGER: Self = Self {
        entity_type: EntityType::Villager,
        width: 0.6,
        height: 1.95,
        eye_height: 1.62,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const VINDICATOR: Self = Self {
        entity_type: EntityType::Vindicator,
        width: 0.6,
        height: 1.95,
        eye_height: 1.6575,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const WANDERING_TRADER: Self = Self {
        entity_type: EntityType::WanderingTrader,
        width: 0.6,
        height: 1.95,
        eye_height: 1.62,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const WARDEN: Self = Self {
        entity_type: EntityType::Warden,
        width: 0.9,
        height: 2.9,
        eye_height: 2.4650002,
        tracking_range: 16,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const WITCH: Self = Self {
        entity_type: EntityType::Witch,
        width: 0.6,
        height: 1.95,
        eye_height: 1.62,
        tracking_range: 8,
        ..MOB_DEFAULTS
    };

    pub const WITHER: Self = Self {
        entity_type: EntityType::Wither,
        width: 0.9,
        height: 3.5,
        eye_height: 2.9750001,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const WITHER_SKELETON: Self = Self {
        entity_type: EntityType::WitherSkeleton,
        width: 0.7,
        height: 2.4,
        eye_height: 2.1,
        tracking_range: 8,
        fire_immune: true,
        ..MOB_DEFAULTS
    };

    pub const ZOGLIN: Self = Self {
        entity_type: EntityType::Zoglin,
        width: 1.3964844,
        height: 1.4,
        eye_height: 1.19,
        tracking_range: 8,
        fire_immune: true,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const ZOMBIE: Self = Self {
        entity_type: EntityType::Zombie,
        width: 0.6,
        height: 1.95,
        eye_height: 1.74,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const ZOMBIE_HORSE: Self = Self {
        entity_type: EntityType::ZombieHorse,
        width: 1.3964844,
        height: 1.6,
        eye_height: 1.52,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const ZOMBIE_NAUTILUS: Self = Self {
        entity_type: EntityType::ZombieNautilus,
        width: 0.875,
        height: 0.95,
        eye_height: 0.2751,
        ..MOB_DEFAULTS
    };

    pub const ZOMBIE_VILLAGER: Self = Self {
        entity_type: EntityType::ZombieVillager,
        width: 0.6,
        height: 1.95,
        eye_height: 1.74,
        tracking_range: 8,
        baby_scale: Some(0.5),
        life: ANIMAL,
        ..MOB_DEFAULTS
    };

    pub const PLAYER: Self = Self {
        entity_type: EntityType::Player,
        width: 0.6,
        height: 1.8,
        eye_height: 1.62,
        tracking_range: 32,
        update_interval: 2,
        tracks_velocity: false,
        life: Life::Living,
        ..MOB_DEFAULTS
    };

    pub const ITEM: Self = Self {
        entity_type: EntityType::Item,
        width: 0.25,
        height: 0.25,
        eye_height: 0.2125,
        tracking_range: 6,
        update_interval: 20,
        client_motion: ClientMotion::Simulated(PreciseSync::LostPrecision),
        life: Life::Lifeless,
        ..MOB_DEFAULTS
    };

    pub const FALLING_BLOCK: Self = Self {
        entity_type: EntityType::FallingBlock,
        width: 0.98,
        height: 0.98,
        eye_height: 0.833_000_06,
        update_interval: 20,
        client_motion: ClientMotion::Simulated(PreciseSync::AfterCollision),
        life: Life::Lifeless,
        ..MOB_DEFAULTS
    };

    pub const TNT: Self = Self {
        entity_type: EntityType::Tnt,
        width: 0.98,
        height: 0.98,
        eye_height: 0.15,
        update_interval: 10,
        client_motion: ClientMotion::Simulated(PreciseSync::AfterCollision),
        fire_immune: true,
        life: Life::Lifeless,
        ..MOB_DEFAULTS
    };

    pub const EXPERIENCE_ORB: Self = Self {
        entity_type: EntityType::ExperienceOrb,
        width: 0.5,
        height: 0.5,
        eye_height: 0.425,
        tracking_range: 6,
        update_interval: 20,
        client_motion: ClientMotion::Simulated(PreciseSync::Never),
        life: Life::Lifeless,
        ..MOB_DEFAULTS
    };

    pub const ALL: [&'static Self; 96] = [
        &Self::PIG,
        &Self::COW,
        &Self::SHEEP,
        &Self::CHICKEN,
        &Self::WOLF,
        &Self::LLAMA,
        &Self::ZOMBIFIED_PIGLIN,
        &Self::FROG,
        &Self::ALLAY,
        &Self::ARMADILLO,
        &Self::AXOLOTL,
        &Self::BAT,
        &Self::BEE,
        &Self::BLAZE,
        &Self::BOGGED,
        &Self::BREEZE,
        &Self::CAMEL,
        &Self::CAMEL_HUSK,
        &Self::CAT,
        &Self::CAVE_SPIDER,
        &Self::COD,
        &Self::COPPER_GOLEM,
        &Self::CREAKING,
        &Self::CREEPER,
        &Self::DOLPHIN,
        &Self::DONKEY,
        &Self::DROWNED,
        &Self::ELDER_GUARDIAN,
        &Self::ENDER_DRAGON,
        &Self::ENDERMAN,
        &Self::ENDERMITE,
        &Self::EVOKER,
        &Self::FOX,
        &Self::FROSTBITE,
        &Self::GHAST,
        &Self::GIANT,
        &Self::GLOW_SQUID,
        &Self::GOAT,
        &Self::GUARDIAN,
        &Self::HAPPY_GHAST,
        &Self::HOGLIN,
        &Self::HORSE,
        &Self::HUSK,
        &Self::ILLUSIONER,
        &Self::IRON_GOLEM,
        &Self::MAGMA_CUBE,
        &Self::MOOSHROOM,
        &Self::MULE,
        &Self::NAUTILUS,
        &Self::OCELOT,
        &Self::PANDA,
        &Self::PARCHED,
        &Self::PARROT,
        &Self::PHANTOM,
        &Self::PIGLIN,
        &Self::PIGLIN_BRUTE,
        &Self::PILLAGER,
        &Self::POLAR_BEAR,
        &Self::PUFFERFISH,
        &Self::RABBIT,
        &Self::RAVAGER,
        &Self::SALMON,
        &Self::SHULKER,
        &Self::SILVERFISH,
        &Self::SKELETON,
        &Self::SKELETON_HORSE,
        &Self::SLIME,
        &Self::SNIFFER,
        &Self::SNOW_GOLEM,
        &Self::SPIDER,
        &Self::SQUID,
        &Self::STRAY,
        &Self::STRIDER,
        &Self::SULFUR_CUBE,
        &Self::TADPOLE,
        &Self::TRADER_LLAMA,
        &Self::TROPICAL_FISH,
        &Self::TURTLE,
        &Self::VEX,
        &Self::VILLAGER,
        &Self::VINDICATOR,
        &Self::WANDERING_TRADER,
        &Self::WARDEN,
        &Self::WITCH,
        &Self::WITHER,
        &Self::WITHER_SKELETON,
        &Self::ZOGLIN,
        &Self::ZOMBIE,
        &Self::ZOMBIE_HORSE,
        &Self::ZOMBIE_NAUTILUS,
        &Self::ZOMBIE_VILLAGER,
        &Self::PLAYER,
        &Self::ITEM,
        &Self::FALLING_BLOCK,
        &Self::TNT,
        &Self::EXPERIENCE_ORB,
    ];

    #[must_use]
    pub const fn of(entity_type: EntityType) -> Option<&'static Self> {
        match entity_type {
            EntityType::Pig => Some(&Self::PIG),
            EntityType::Cow => Some(&Self::COW),
            EntityType::Sheep => Some(&Self::SHEEP),
            EntityType::Chicken => Some(&Self::CHICKEN),
            EntityType::Wolf => Some(&Self::WOLF),
            EntityType::Llama => Some(&Self::LLAMA),
            EntityType::ZombifiedPiglin => Some(&Self::ZOMBIFIED_PIGLIN),
            EntityType::Frog => Some(&Self::FROG),
            EntityType::Allay => Some(&Self::ALLAY),
            EntityType::Armadillo => Some(&Self::ARMADILLO),
            EntityType::Axolotl => Some(&Self::AXOLOTL),
            EntityType::Bat => Some(&Self::BAT),
            EntityType::Bee => Some(&Self::BEE),
            EntityType::Blaze => Some(&Self::BLAZE),
            EntityType::Bogged => Some(&Self::BOGGED),
            EntityType::Breeze => Some(&Self::BREEZE),
            EntityType::Camel => Some(&Self::CAMEL),
            EntityType::CamelHusk => Some(&Self::CAMEL_HUSK),
            EntityType::Cat => Some(&Self::CAT),
            EntityType::CaveSpider => Some(&Self::CAVE_SPIDER),
            EntityType::Cod => Some(&Self::COD),
            EntityType::CopperGolem => Some(&Self::COPPER_GOLEM),
            EntityType::Creaking => Some(&Self::CREAKING),
            EntityType::Creeper => Some(&Self::CREEPER),
            EntityType::Dolphin => Some(&Self::DOLPHIN),
            EntityType::Donkey => Some(&Self::DONKEY),
            EntityType::Drowned => Some(&Self::DROWNED),
            EntityType::ElderGuardian => Some(&Self::ELDER_GUARDIAN),
            EntityType::EnderDragon => Some(&Self::ENDER_DRAGON),
            EntityType::Enderman => Some(&Self::ENDERMAN),
            EntityType::Endermite => Some(&Self::ENDERMITE),
            EntityType::Evoker => Some(&Self::EVOKER),
            EntityType::Fox => Some(&Self::FOX),
            EntityType::Frostbite => Some(&Self::FROSTBITE),
            EntityType::Ghast => Some(&Self::GHAST),
            EntityType::Giant => Some(&Self::GIANT),
            EntityType::GlowSquid => Some(&Self::GLOW_SQUID),
            EntityType::Goat => Some(&Self::GOAT),
            EntityType::Guardian => Some(&Self::GUARDIAN),
            EntityType::HappyGhast => Some(&Self::HAPPY_GHAST),
            EntityType::Hoglin => Some(&Self::HOGLIN),
            EntityType::Horse => Some(&Self::HORSE),
            EntityType::Husk => Some(&Self::HUSK),
            EntityType::Illusioner => Some(&Self::ILLUSIONER),
            EntityType::IronGolem => Some(&Self::IRON_GOLEM),
            EntityType::MagmaCube => Some(&Self::MAGMA_CUBE),
            EntityType::Mooshroom => Some(&Self::MOOSHROOM),
            EntityType::Mule => Some(&Self::MULE),
            EntityType::Nautilus => Some(&Self::NAUTILUS),
            EntityType::Ocelot => Some(&Self::OCELOT),
            EntityType::Panda => Some(&Self::PANDA),
            EntityType::Parched => Some(&Self::PARCHED),
            EntityType::Parrot => Some(&Self::PARROT),
            EntityType::Phantom => Some(&Self::PHANTOM),
            EntityType::Piglin => Some(&Self::PIGLIN),
            EntityType::PiglinBrute => Some(&Self::PIGLIN_BRUTE),
            EntityType::Pillager => Some(&Self::PILLAGER),
            EntityType::PolarBear => Some(&Self::POLAR_BEAR),
            EntityType::Pufferfish => Some(&Self::PUFFERFISH),
            EntityType::Rabbit => Some(&Self::RABBIT),
            EntityType::Ravager => Some(&Self::RAVAGER),
            EntityType::Salmon => Some(&Self::SALMON),
            EntityType::Shulker => Some(&Self::SHULKER),
            EntityType::Silverfish => Some(&Self::SILVERFISH),
            EntityType::Skeleton => Some(&Self::SKELETON),
            EntityType::SkeletonHorse => Some(&Self::SKELETON_HORSE),
            EntityType::Slime => Some(&Self::SLIME),
            EntityType::Sniffer => Some(&Self::SNIFFER),
            EntityType::SnowGolem => Some(&Self::SNOW_GOLEM),
            EntityType::Spider => Some(&Self::SPIDER),
            EntityType::Squid => Some(&Self::SQUID),
            EntityType::Stray => Some(&Self::STRAY),
            EntityType::Strider => Some(&Self::STRIDER),
            EntityType::SulfurCube => Some(&Self::SULFUR_CUBE),
            EntityType::Tadpole => Some(&Self::TADPOLE),
            EntityType::TraderLlama => Some(&Self::TRADER_LLAMA),
            EntityType::TropicalFish => Some(&Self::TROPICAL_FISH),
            EntityType::Turtle => Some(&Self::TURTLE),
            EntityType::Vex => Some(&Self::VEX),
            EntityType::Villager => Some(&Self::VILLAGER),
            EntityType::Vindicator => Some(&Self::VINDICATOR),
            EntityType::WanderingTrader => Some(&Self::WANDERING_TRADER),
            EntityType::Warden => Some(&Self::WARDEN),
            EntityType::Witch => Some(&Self::WITCH),
            EntityType::Wither => Some(&Self::WITHER),
            EntityType::WitherSkeleton => Some(&Self::WITHER_SKELETON),
            EntityType::Zoglin => Some(&Self::ZOGLIN),
            EntityType::Zombie => Some(&Self::ZOMBIE),
            EntityType::ZombieHorse => Some(&Self::ZOMBIE_HORSE),
            EntityType::ZombieNautilus => Some(&Self::ZOMBIE_NAUTILUS),
            EntityType::ZombieVillager => Some(&Self::ZOMBIE_VILLAGER),
            EntityType::Player => Some(&Self::PLAYER),
            EntityType::Item => Some(&Self::ITEM),
            EntityType::FallingBlock => Some(&Self::FALLING_BLOCK),
            EntityType::Tnt => Some(&Self::TNT),
            EntityType::ExperienceOrb => Some(&Self::EXPERIENCE_ORB),
            _ => None,
        }
    }

    #[must_use]
    pub const fn attribute_bases(&self) -> Option<AttributeBases> {
        crate::attribute_bases(self.entity_type)
    }

    #[must_use]
    pub const fn mob(&self) -> Option<&MobData> {
        match &self.life {
            Life::Mob(mob) => Some(mob),
            Life::Living | Life::Lifeless => None,
        }
    }

    #[must_use]
    pub const fn is_living(&self) -> bool {
        matches!(self.life, Life::Living | Life::Mob(_))
    }

    #[must_use]
    pub const fn is_ageable(&self) -> bool {
        matches!(self.life, Life::Mob(MobData { ageable: true, .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn of_finds_each_row_of_all() {
        for row in EntityTypeData::ALL {
            assert_eq!(
                EntityTypeData::of(row.entity_type),
                Some(row),
                "{:?}",
                row.entity_type
            );
        }
        let listed = EntityType::ALL
            .iter()
            .filter(|entity_type| EntityTypeData::of(**entity_type).is_some())
            .count();
        assert_eq!(
            listed,
            EntityTypeData::ALL.len(),
            "ALL misses a row of `of`"
        );
    }
}
