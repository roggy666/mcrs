use mcrs_minecraft_item_stack::AttackAnimation;
use mcrs_minecraft_item_stack::AttributeModifiers;
use mcrs_minecraft_item_stack::AxolotlVariant;
use mcrs_minecraft_item_stack::BannerPatterns;
use mcrs_minecraft_item_stack::Bees;
use mcrs_minecraft_item_stack::BlockState;
use mcrs_minecraft_item_stack::Builtin;
use mcrs_minecraft_item_stack::BundleContents;
use mcrs_minecraft_item_stack::CatCollar;
use mcrs_minecraft_item_stack::CatSoundVariant;
use mcrs_minecraft_item_stack::CatVariant;
use mcrs_minecraft_item_stack::ChargedProjectiles;
use mcrs_minecraft_item_stack::ChickenSoundVariant;
use mcrs_minecraft_item_stack::ChickenVariant;
use mcrs_minecraft_item_stack::Container;
use mcrs_minecraft_item_stack::ContainerLoot;
use mcrs_minecraft_item_stack::CowSoundVariant;
use mcrs_minecraft_item_stack::CowVariant;
use mcrs_minecraft_item_stack::CushionColor;
use mcrs_minecraft_item_stack::CustomData;
use mcrs_minecraft_item_stack::CustomModelData;
use mcrs_minecraft_item_stack::CustomName;
use mcrs_minecraft_item_stack::Damage;
use mcrs_minecraft_item_stack::DyedColor;
use mcrs_minecraft_item_stack::Enchantable;
use mcrs_minecraft_item_stack::EnchantmentGlintOverride;
use mcrs_minecraft_item_stack::Enchantments;
use mcrs_minecraft_item_stack::FireworkExplosion;
use mcrs_minecraft_item_stack::Fireworks;
use mcrs_minecraft_item_stack::Food;
use mcrs_minecraft_item_stack::FoxVariant;
use mcrs_minecraft_item_stack::FrogVariant;
use mcrs_minecraft_item_stack::HorseVariant;
use mcrs_minecraft_item_stack::Instrument;
use mcrs_minecraft_item_stack::InteractAnimation;
use mcrs_minecraft_item_stack::ItemComponent;
use mcrs_minecraft_item_stack::ItemModel;
use mcrs_minecraft_item_stack::ItemName;
use mcrs_minecraft_item_stack::ItemStack;
use mcrs_minecraft_item_stack::JukeboxPlayable;
use mcrs_minecraft_item_stack::LlamaVariant;
use mcrs_minecraft_item_stack::LodestoneTracker;
use mcrs_minecraft_item_stack::Lore;
use mcrs_minecraft_item_stack::MapDecorations;
use mcrs_minecraft_item_stack::MapId;
use mcrs_minecraft_item_stack::MaxDamage;
use mcrs_minecraft_item_stack::MaxStackSize;
use mcrs_minecraft_item_stack::MooshroomVariant;
use mcrs_minecraft_item_stack::OminousBottleAmplifier;
use mcrs_minecraft_item_stack::PaintingVariant;
use mcrs_minecraft_item_stack::ParrotVariant;
use mcrs_minecraft_item_stack::PiercingWeapon;
use mcrs_minecraft_item_stack::PigSoundVariant;
use mcrs_minecraft_item_stack::PigVariant;
use mcrs_minecraft_item_stack::PotionContents;
use mcrs_minecraft_item_stack::Profile;
use mcrs_minecraft_item_stack::RabbitVariant;
use mcrs_minecraft_item_stack::Rarity;
use mcrs_minecraft_item_stack::RepairCost;
use mcrs_minecraft_item_stack::SalmonSize;
use mcrs_minecraft_item_stack::SheepColor;
use mcrs_minecraft_item_stack::ShulkerColor;
use mcrs_minecraft_item_stack::StoredEnchantments;
use mcrs_minecraft_item_stack::SuspiciousStewEffects;
use mcrs_minecraft_item_stack::Tool;
use mcrs_minecraft_item_stack::TooltipDisplay;
use mcrs_minecraft_item_stack::Trim;
use mcrs_minecraft_item_stack::TropicalFishBaseColor;
use mcrs_minecraft_item_stack::TropicalFishPattern;
use mcrs_minecraft_item_stack::TropicalFishPatternColor;
use mcrs_minecraft_item_stack::Unbreakable;
use mcrs_minecraft_item_stack::VillagerVariant;
use mcrs_minecraft_item_stack::WolfCollar;
use mcrs_minecraft_item_stack::WolfSoundVariant;
use mcrs_minecraft_item_stack::WolfVariant;
use mcrs_minecraft_item_stack::WritableBookContent;
use mcrs_minecraft_item_stack::WrittenBookContent;
use mcrs_minecraft_item_stack::ZombieNautilusVariant;
use mcrs_minecraft_item_table::ItemTable;

use mcrs_minecraft_item::data_component_type_keys::DataComponentType;

#[derive(Clone, Copy, Debug)]
pub(crate) enum ComponentHolder<'a> {
    Empty,
    Stack {
        stack: &'a ItemStack,
        table: &'a ItemTable,
    },
}

impl<'a> ComponentHolder<'a> {
    pub(crate) fn effective<T: ItemComponent + 'static>(self) -> Option<&'a T> {
        match self {
            Self::Empty => None,
            Self::Stack { stack, table } => table.effective(stack),
        }
    }

    #[expect(clippy::too_many_lines, reason = "one arm per component type")]
    pub(crate) fn has(self, kind: DataComponentType) -> bool {
        match kind {
            DataComponentType::MaxStackSize => self.effective::<MaxStackSize>().is_some(),
            DataComponentType::MaxDamage => self.effective::<MaxDamage>().is_some(),
            DataComponentType::Damage => self.effective::<Damage>().is_some(),
            DataComponentType::Unbreakable => self.effective::<Unbreakable>().is_some(),
            DataComponentType::CustomName => self.effective::<CustomName>().is_some(),
            DataComponentType::ItemName => self.effective::<ItemName>().is_some(),
            DataComponentType::ItemModel => self.effective::<ItemModel>().is_some(),
            DataComponentType::Lore => self.effective::<Lore>().is_some(),
            DataComponentType::Rarity => self.effective::<Rarity>().is_some(),
            DataComponentType::Enchantments => self.effective::<Enchantments>().is_some(),
            DataComponentType::AttributeModifiers => {
                self.effective::<AttributeModifiers>().is_some()
            }
            DataComponentType::TooltipDisplay => self.effective::<TooltipDisplay>().is_some(),
            DataComponentType::RepairCost => self.effective::<RepairCost>().is_some(),
            DataComponentType::Food => self.effective::<Food>().is_some(),
            DataComponentType::Tool => self.effective::<Tool>().is_some(),
            DataComponentType::MapId => self.effective::<MapId>().is_some(),
            DataComponentType::MapDecorations => self.effective::<MapDecorations>().is_some(),
            DataComponentType::Profile => self.effective::<Profile>().is_some(),
            DataComponentType::DyedColor => self.effective::<DyedColor>().is_some(),
            DataComponentType::CustomModelData => self.effective::<CustomModelData>().is_some(),
            DataComponentType::EnchantmentGlintOverride => {
                self.effective::<EnchantmentGlintOverride>().is_some()
            }
            DataComponentType::BlockState => self.effective::<BlockState>().is_some(),
            DataComponentType::LodestoneTracker => self.effective::<LodestoneTracker>().is_some(),
            DataComponentType::Trim => self.effective::<Trim>().is_some(),
            DataComponentType::PotionContents => self.effective::<PotionContents>().is_some(),
            DataComponentType::CustomData => self.effective::<CustomData>().is_some(),
            DataComponentType::SuspiciousStewEffects => {
                self.effective::<SuspiciousStewEffects>().is_some()
            }
            DataComponentType::OminousBottleAmplifier => {
                self.effective::<OminousBottleAmplifier>().is_some()
            }
            DataComponentType::Instrument => self.effective::<Instrument>().is_some(),
            DataComponentType::BannerPatterns => self.effective::<BannerPatterns>().is_some(),
            DataComponentType::Bees => self.effective::<Bees>().is_some(),
            DataComponentType::BundleContents => self.effective::<BundleContents>().is_some(),
            DataComponentType::ChargedProjectiles => {
                self.effective::<ChargedProjectiles>().is_some()
            }
            DataComponentType::Container => self.effective::<Container>().is_some(),
            DataComponentType::ContainerLoot => self.effective::<ContainerLoot>().is_some(),
            DataComponentType::FireworkExplosion => self.effective::<FireworkExplosion>().is_some(),
            DataComponentType::Fireworks => self.effective::<Fireworks>().is_some(),
            DataComponentType::StoredEnchantments => {
                self.effective::<StoredEnchantments>().is_some()
            }
            DataComponentType::PaintingVariant => self.effective::<PaintingVariant>().is_some(),
            DataComponentType::PiercingWeapon => self.effective::<PiercingWeapon>().is_some(),
            DataComponentType::AttackAnimation => self.effective::<AttackAnimation>().is_some(),
            DataComponentType::InteractAnimation => self.effective::<InteractAnimation>().is_some(),
            DataComponentType::Enchantable => self.effective::<Enchantable>().is_some(),
            DataComponentType::WritableBookContent => {
                self.effective::<WritableBookContent>().is_some()
            }
            DataComponentType::WrittenBookContent => {
                self.effective::<WrittenBookContent>().is_some()
            }
            DataComponentType::JukeboxPlayable => self.effective::<JukeboxPlayable>().is_some(),
            DataComponentType::VillagerVariant => self.effective::<VillagerVariant>().is_some(),
            DataComponentType::WolfVariant => self.effective::<WolfVariant>().is_some(),
            DataComponentType::WolfSoundVariant => self.effective::<WolfSoundVariant>().is_some(),
            DataComponentType::WolfCollar => self.effective::<WolfCollar>().is_some(),
            DataComponentType::FoxVariant => self.effective::<FoxVariant>().is_some(),
            DataComponentType::SalmonSize => self.effective::<SalmonSize>().is_some(),
            DataComponentType::ParrotVariant => self.effective::<ParrotVariant>().is_some(),
            DataComponentType::TropicalFishPattern => {
                self.effective::<TropicalFishPattern>().is_some()
            }
            DataComponentType::TropicalFishBaseColor => {
                self.effective::<TropicalFishBaseColor>().is_some()
            }
            DataComponentType::TropicalFishPatternColor => {
                self.effective::<TropicalFishPatternColor>().is_some()
            }
            DataComponentType::MooshroomVariant => self.effective::<MooshroomVariant>().is_some(),
            DataComponentType::RabbitVariant => self.effective::<RabbitVariant>().is_some(),
            DataComponentType::PigVariant => self.effective::<PigVariant>().is_some(),
            DataComponentType::PigSoundVariant => self.effective::<PigSoundVariant>().is_some(),
            DataComponentType::CowVariant => self.effective::<CowVariant>().is_some(),
            DataComponentType::CowSoundVariant => self.effective::<CowSoundVariant>().is_some(),
            DataComponentType::ChickenVariant => self.effective::<ChickenVariant>().is_some(),
            DataComponentType::ChickenSoundVariant => {
                self.effective::<ChickenSoundVariant>().is_some()
            }
            DataComponentType::ZombieNautilusVariant => {
                self.effective::<ZombieNautilusVariant>().is_some()
            }
            DataComponentType::FrogVariant => self.effective::<FrogVariant>().is_some(),
            DataComponentType::HorseVariant => self.effective::<HorseVariant>().is_some(),
            DataComponentType::LlamaVariant => self.effective::<LlamaVariant>().is_some(),
            DataComponentType::AxolotlVariant => self.effective::<AxolotlVariant>().is_some(),
            DataComponentType::CatVariant => self.effective::<CatVariant>().is_some(),
            DataComponentType::CatSoundVariant => self.effective::<CatSoundVariant>().is_some(),
            DataComponentType::CatCollar => self.effective::<CatCollar>().is_some(),
            DataComponentType::SheepColor => self.effective::<SheepColor>().is_some(),
            DataComponentType::ShulkerColor => self.effective::<ShulkerColor>().is_some(),
            DataComponentType::CushionColor => self.effective::<CushionColor>().is_some(),
            _ => false,
        }
    }

    pub(crate) fn has_equal(self, expected: &Builtin) -> bool {
        match expected {
            Builtin::MaxStackSize(value) => self.effective() == Some(value),
            Builtin::MaxDamage(value) => self.effective() == Some(value),
            Builtin::Damage(value) => self.effective() == Some(value),
            Builtin::Unbreakable(value) => self.effective() == Some(value),
            Builtin::CustomName(value) => self.effective() == Some(value),
            Builtin::ItemName(value) => self.effective() == Some(value),
            Builtin::ItemModel(value) => self.effective() == Some(value),
            Builtin::Lore(value) => self.effective() == Some(value),
            Builtin::Rarity(value) => self.effective() == Some(value),
            Builtin::Enchantments(value) => self.effective() == Some(value),
            Builtin::AttributeModifiers(value) => self.effective() == Some(value),
            Builtin::TooltipDisplay(value) => self.effective() == Some(value),
            Builtin::RepairCost(value) => self.effective() == Some(value),
            Builtin::Food(value) => self.effective() == Some(value),
            Builtin::Tool(value) => self.effective() == Some(value),
            Builtin::MapId(value) => self.effective() == Some(value),
            Builtin::MapDecorations(value) => self.effective() == Some(value),
            Builtin::Profile(value) => self.effective() == Some(value),
            Builtin::DyedColor(value) => self.effective() == Some(value),
            Builtin::CustomModelData(value) => self.effective() == Some(value),
            Builtin::EnchantmentGlintOverride(value) => self.effective() == Some(value),
            Builtin::BlockState(value) => self.effective() == Some(value),
            Builtin::LodestoneTracker(value) => self.effective() == Some(value),
            Builtin::Trim(value) => self.effective() == Some(value),
            Builtin::PotionContents(value) => self.effective() == Some(value),
            Builtin::CustomData(value) => self.effective() == Some(value),
            Builtin::SuspiciousStewEffects(value) => self.effective() == Some(value),
            Builtin::OminousBottleAmplifier(value) => self.effective() == Some(value),
            Builtin::Instrument(value) => self.effective() == Some(value),
            Builtin::BannerPatterns(value) => self.effective() == Some(value),
            Builtin::Bees(value) => self.effective() == Some(value),
            Builtin::BundleContents(value) => self.effective() == Some(value),
            Builtin::ChargedProjectiles(value) => self.effective() == Some(value),
            Builtin::Container(value) => self.effective() == Some(value),
            Builtin::ContainerLoot(value) => self.effective() == Some(value),
            Builtin::FireworkExplosion(value) => self.effective() == Some(value),
            Builtin::Fireworks(value) => self.effective() == Some(value),
            Builtin::StoredEnchantments(value) => self.effective() == Some(value),
            Builtin::PaintingVariant(value) => self.effective() == Some(value),
            Builtin::PiercingWeapon(value) => self.effective() == Some(value),
            Builtin::AttackAnimation(value) => self.effective() == Some(value),
            Builtin::InteractAnimation(value) => self.effective() == Some(value),
            Builtin::Enchantable(value) => self.effective() == Some(value),
            Builtin::WritableBookContent(value) => self.effective() == Some(value),
            Builtin::WrittenBookContent(value) => self.effective() == Some(value),
            Builtin::JukeboxPlayable(value) => self.effective() == Some(value),
            Builtin::VillagerVariant(value) => self.effective() == Some(value),
            Builtin::WolfVariant(value) => self.effective() == Some(value),
            Builtin::WolfSoundVariant(value) => self.effective() == Some(value),
            Builtin::WolfCollar(value) => self.effective() == Some(value),
            Builtin::FoxVariant(value) => self.effective() == Some(value),
            Builtin::SalmonSize(value) => self.effective() == Some(value),
            Builtin::ParrotVariant(value) => self.effective() == Some(value),
            Builtin::TropicalFishPattern(value) => self.effective() == Some(value),
            Builtin::TropicalFishBaseColor(value) => self.effective() == Some(value),
            Builtin::TropicalFishPatternColor(value) => self.effective() == Some(value),
            Builtin::MooshroomVariant(value) => self.effective() == Some(value),
            Builtin::RabbitVariant(value) => self.effective() == Some(value),
            Builtin::PigVariant(value) => self.effective() == Some(value),
            Builtin::PigSoundVariant(value) => self.effective() == Some(value),
            Builtin::CowVariant(value) => self.effective() == Some(value),
            Builtin::CowSoundVariant(value) => self.effective() == Some(value),
            Builtin::ChickenVariant(value) => self.effective() == Some(value),
            Builtin::ChickenSoundVariant(value) => self.effective() == Some(value),
            Builtin::ZombieNautilusVariant(value) => self.effective() == Some(value),
            Builtin::FrogVariant(value) => self.effective() == Some(value),
            Builtin::HorseVariant(value) => self.effective() == Some(value),
            Builtin::LlamaVariant(value) => self.effective() == Some(value),
            Builtin::AxolotlVariant(value) => self.effective() == Some(value),
            Builtin::CatVariant(value) => self.effective() == Some(value),
            Builtin::CatSoundVariant(value) => self.effective() == Some(value),
            Builtin::CatCollar(value) => self.effective() == Some(value),
            Builtin::SheepColor(value) => self.effective() == Some(value),
            Builtin::ShulkerColor(value) => self.effective() == Some(value),
            Builtin::CushionColor(value) => self.effective() == Some(value),
            _ => false,
        }
    }
}
