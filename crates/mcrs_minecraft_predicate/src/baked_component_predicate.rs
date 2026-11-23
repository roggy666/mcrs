use fixedbitset::FixedBitSet;
use mcrs_minecraft_item_stack::AttributeModifiers;
use mcrs_minecraft_item_stack::Damage;
use mcrs_minecraft_item_stack::Enchantments;
use mcrs_minecraft_item_stack::JukeboxPlayable;
use mcrs_minecraft_item_stack::MaxDamage;
use mcrs_minecraft_item_stack::RegistryValue;
use mcrs_minecraft_item_stack::Trim;
use mcrs_minecraft_item_stack::VillagerVariant;
use mcrs_minecraft_registries::Registries;
use mcrs_minecraft_value_provider::bounds::MinMaxBounds;

use crate::baked_collection::BakedCollection;
use crate::baked_enchantment_predicate::BakedEnchantmentPredicate;
use crate::baked_modifier_predicate::BakedModifierPredicate;
use crate::component_holder::ComponentHolder;
use crate::id_bits::id_bits;
use crate::predicate_issue::PredicateIssue;
use mcrs_minecraft_item::component::predicate::ComponentPredicate;
use mcrs_minecraft_item::component::predicate::ComponentPredicateEntry;
use mcrs_minecraft_item::component::predicate::DamagePredicate;
use mcrs_minecraft_item::data_component_type_keys::DataComponentType;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BakedComponentPredicate {
    Present(DataComponentType),
    Damage(DamagePredicate),
    Enchantments(Box<[BakedEnchantmentPredicate]>),
    AttributeModifiers(Option<BakedCollection<BakedModifierPredicate>>),
    Trim {
        material: Option<FixedBitSet>,
        pattern: Option<FixedBitSet>,
    },
    JukeboxPlayable(Option<FixedBitSet>),
    VillagerVariant(FixedBitSet),
    Always,
    Never,
}

impl BakedComponentPredicate {
    pub(crate) fn bake(
        entry: &ComponentPredicateEntry,
        registries: &Registries,
    ) -> Result<Self, PredicateIssue> {
        let predicate = match entry {
            ComponentPredicateEntry::AnyValue(kind) => return Ok(Self::Present(*kind)),
            ComponentPredicateEntry::Typed(predicate) => predicate,
        };
        Ok(match predicate {
            ComponentPredicate::Damage(damage) => Self::Damage(damage.clone()),
            ComponentPredicate::Enchantments(list) => Self::Enchantments(
                list.0
                    .iter()
                    .map(|element| BakedEnchantmentPredicate::bake(element, registries))
                    .collect::<Result<_, _>>()?,
            ),
            ComponentPredicate::AttributeModifiers(predicate) => Self::AttributeModifiers(
                predicate
                    .modifiers
                    .as_ref()
                    .map(|modifiers| {
                        BakedCollection::bake(modifiers, |element| {
                            BakedModifierPredicate::bake(element, registries)
                        })
                    })
                    .transpose()?,
            ),
            ComponentPredicate::Trim(trim) => Self::Trim {
                material: (trim.material.as_ref())
                    .map(|set| id_bits(set, registries, PredicateIssue::UnloadedTrimMaterialTags))
                    .transpose()?,
                pattern: (trim.pattern.as_ref())
                    .map(|set| id_bits(set, registries, PredicateIssue::UnloadedTrimPatternTags))
                    .transpose()?,
            },
            ComponentPredicate::JukeboxPlayable(playable) => Self::JukeboxPlayable(
                (playable.song.as_ref())
                    .map(|set| id_bits(set, registries, PredicateIssue::UnloadedJukeboxSongTags))
                    .transpose()?,
            ),
            ComponentPredicate::VillagerVariant(set) => Self::VillagerVariant(id_bits(
                set,
                registries,
                PredicateIssue::UnloadedVillagerTypeTags,
            )?),
            ComponentPredicate::CustomData(expected) if expected.0.is_empty() => Self::Always,
            ComponentPredicate::CustomData(_)
            | ComponentPredicate::StoredEnchantments(_)
            | ComponentPredicate::PotionContents(_)
            | ComponentPredicate::Container(_)
            | ComponentPredicate::BundleContents(_)
            | ComponentPredicate::FireworkExplosion(_)
            | ComponentPredicate::Fireworks(_)
            | ComponentPredicate::WritableBookContent(_)
            | ComponentPredicate::WrittenBookContent(_) => Self::Never,
        })
    }

    pub(crate) fn matches(&self, holder: ComponentHolder<'_>) -> bool {
        match self {
            Self::Present(kind) => holder.has(*kind),
            Self::Damage(predicate) => holder.effective::<Damage>().is_some_and(|damage| {
                let max = holder.effective::<MaxDamage>().map_or(0, |max| max.0);
                let durability = i64::from(max) - i64::from(damage.0);
                let bounds = MinMaxBounds {
                    min: predicate.durability.min.map(i64::from),
                    max: predicate.durability.max.map(i64::from),
                };
                bounds.matches(durability) && predicate.damage.matches(damage.0)
            }),
            Self::Enchantments(elements) => holder
                .effective::<Enchantments>()
                .is_some_and(|component| elements.iter().all(|element| element.matches(component))),
            Self::AttributeModifiers(modifiers) => holder
                .effective::<AttributeModifiers>()
                .is_some_and(|component| {
                    modifiers.as_ref().is_none_or(|collection| {
                        collection.matches(&component.modifiers, BakedModifierPredicate::matches)
                    })
                }),
            Self::Trim { material, pattern } => holder.effective::<Trim>().is_some_and(|trim| {
                names_entry(material.as_ref(), &trim.material, |id| id.0)
                    && names_entry(pattern.as_ref(), &trim.pattern, |id| id.0)
            }),
            Self::JukeboxPlayable(songs) => {
                holder
                    .effective::<JukeboxPlayable>()
                    .is_some_and(|playable| {
                        songs
                            .as_ref()
                            .is_none_or(|songs| songs.contains(usize::from(playable.0.0)))
                    })
            }
            Self::VillagerVariant(types) => holder
                .effective::<VillagerVariant>()
                .is_some_and(|variant| types.contains(usize::from(variant.0.0))),
            Self::Always => true,
            Self::Never => false,
        }
    }
}

fn names_entry<I: Copy, T>(
    set: Option<&FixedBitSet>,
    value: &RegistryValue<I, T>,
    id: impl Fn(I) -> u16,
) -> bool {
    match (set, value) {
        (None, _) => true,
        (Some(set), RegistryValue::Reference(reference)) => {
            set.contains(usize::from(id(*reference)))
        }
        (Some(_), RegistryValue::Inline(_)) => false,
    }
}
