use mcrs_minecraft_brain::ItemKind;
use mcrs_minecraft_brain::ItemTag;
use mcrs_minecraft_brain::ItemTest;
use mcrs_minecraft_item_key::item_keys::Item;
use mcrs_minecraft_item_key::item_tag_keys;
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
];

#[must_use]
pub const fn item(item: Item) -> ItemKind {
    ItemKind(item.id().number())
}

pub const CARROT_ON_A_STICK: ItemTest = ItemTest::Item(item(Item::CarrotOnAStick));
