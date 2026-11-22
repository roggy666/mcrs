use mcrs_minecraft_brain::ItemKind;
use mcrs_minecraft_brain::ItemSet;
use mcrs_minecraft_brain::ItemTables;
use mcrs_minecraft_brain::ItemTag;
use mcrs_minecraft_brain::ItemTest;
use mcrs_minecraft_item_key::item_keys::Item;
use mcrs_minecraft_item_key::item_tag_keys;
use mcrs_minecraft_registries::Registries;
use mcrs_minecraft_resource::TagKey;

#[derive(Clone, Copy, Debug)]
pub struct ItemSetSpec {
    pub tag: Option<TagKey<'static, Item>>,
    pub items: &'static [Item],
}

pub const PIG_FOOD: ItemTag = ItemTag(0);
pub const COW_FOOD: ItemTag = ItemTag(1);
pub const SHEEP_FOOD: ItemTag = ItemTag(2);
pub const CHICKEN_FOOD: ItemTag = ItemTag(3);
pub const WOLF_FOOD: ItemTag = ItemTag(4);
pub const WOLF_INTERESTING: ItemTag = ItemTag(5);
pub const LLAMA_TEMPT_ITEMS: ItemTag = ItemTag(6);
pub const CAT_FOOD: ItemTag = ItemTag(7);
pub const OCELOT_FOOD: ItemTag = ItemTag(8);
pub const RABBIT_FOOD: ItemTag = ItemTag(9);
pub const PANDA_FOOD: ItemTag = ItemTag(10);
pub const HORSE_TEMPT_ITEMS: ItemTag = ItemTag(11);
pub const TURTLE_FOOD: ItemTag = ItemTag(12);
pub const STRIDER_TEMPT_ITEMS: ItemTag = ItemTag(13);
pub const BEE_FOOD: ItemTag = ItemTag(14);
pub const PARROT_FOOD: ItemTag = ItemTag(15);

pub const ITEM_SETS: &[ItemSetSpec] = &[
    ItemSetSpec {
        tag: Some(item_tag_keys::PIG_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::COW_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::SHEEP_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::CHICKEN_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::WOLF_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::WOLF_FOOD),
        items: &[Item::Bone],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::LLAMA_TEMPT_ITEMS),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::CAT_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::OCELOT_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::RABBIT_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::PANDA_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::HORSE_TEMPT_ITEMS),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::TURTLE_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::STRIDER_TEMPT_ITEMS),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::BEE_FOOD),
        items: &[],
    },
    ItemSetSpec {
        tag: Some(item_tag_keys::PARROT_FOOD),
        items: &[],
    },
];

#[must_use]
pub const fn item(item: Item) -> ItemKind {
    ItemKind(item.id().number())
}

pub const CARROT_ON_A_STICK: ItemTest = ItemTest::Item(item(Item::CarrotOnAStick));

#[must_use]
pub fn item_tables(registries: &Registries) -> ItemTables {
    let tags = registries.tags::<Item>();
    ItemTables::new(ITEM_SETS.iter().map(|spec| {
        let mut set = ItemSet::new(spec.items.iter().map(|listed| item(*listed)));
        if let (Some(tags), Some(tag)) = (tags.as_ref(), spec.tag)
            && let Some(id) = tags.id(tag)
        {
            for member in tags.members(id) {
                set.insert(ItemKind(member.number()));
            }
        }
        set
    }))
}
