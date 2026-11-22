use mcrs_minecraft_block::block_keys::Block;
use mcrs_minecraft_block::definition::placement_filter::FaceTest;
use mcrs_minecraft_block::definition::{BlockDefinitions, BlockEntry};
use mcrs_minecraft_core::Direction;
use mcrs_minecraft_registries::Tags;
use mcrs_minecraft_resource::{Id, TagId};
use mcrs_voxel::BlockStateId;

use crate::block_connections::{
    BARS, COVERS_ARM, COVERS_POST, Digit, EXCEPTED, FULL_FACE, Family, GATE_ALONG_X, GATE_ALONG_Z,
    Kind, OTHER_FENCE, PANE, POST_OVERRIDE, Sides, Traits, WALL, WOODEN_FENCE, horizontal_bit,
};
use crate::face_test;

pub(crate) struct TagSet {
    pub(crate) wooden_fences: Option<TagId<Block>>,
    pub(crate) fences: Option<TagId<Block>>,
    pub(crate) fence_gates: Option<TagId<Block>>,
    pub(crate) walls: Option<TagId<Block>>,
    pub(crate) bars: Option<TagId<Block>>,
    pub(crate) shulker_boxes: Option<TagId<Block>>,
    pub(crate) leaves: Option<TagId<Block>>,
    pub(crate) wall_post_override: Option<TagId<Block>>,
}

fn in_tag(tags: Option<&Tags<Block>>, tag: Option<TagId<Block>>, block: Id<Block>) -> bool {
    match (tags, tag) {
        (Some(tags), Some(tag)) => tags.contains(tag, block),
        _ => false,
    }
}

fn is_pane(block: Id<Block>) -> bool {
    const PANES: [Block; 17] = [
        Block::GlassPane,
        Block::WhiteStainedGlassPane,
        Block::OrangeStainedGlassPane,
        Block::MagentaStainedGlassPane,
        Block::LightBlueStainedGlassPane,
        Block::YellowStainedGlassPane,
        Block::LimeStainedGlassPane,
        Block::PinkStainedGlassPane,
        Block::GrayStainedGlassPane,
        Block::LightGrayStainedGlassPane,
        Block::CyanStainedGlassPane,
        Block::PurpleStainedGlassPane,
        Block::BlueStainedGlassPane,
        Block::BrownStainedGlassPane,
        Block::GreenStainedGlassPane,
        Block::RedStainedGlassPane,
        Block::BlackStainedGlassPane,
    ];
    PANES.iter().any(|pane| pane.id() == block)
}

fn is_excepted_block(block: Id<Block>) -> bool {
    const EXCEPTED_BLOCKS: [Block; 5] = [
        Block::Barrier,
        Block::CarvedPumpkin,
        Block::JackOLantern,
        Block::Melon,
        Block::Pumpkin,
    ];
    EXCEPTED_BLOCKS
        .iter()
        .any(|excepted| excepted.id() == block)
}

pub(crate) fn kind_of(block: Id<Block>, tags: Option<&Tags<Block>>, sets: &TagSet) -> Option<Kind> {
    if in_tag(tags, sets.fences, block) {
        Some(Kind::Fence)
    } else if is_pane(block) || in_tag(tags, sets.bars, block) {
        Some(Kind::PaneOrBars)
    } else if in_tag(tags, sets.walls, block) {
        Some(Kind::Wall)
    } else if in_tag(tags, sets.fence_gates, block) {
        Some(Kind::Gate)
    } else {
        None
    }
}

pub(crate) fn family_of(entry: &BlockEntry, kind: Kind) -> Family {
    let side_values: [&str; 3] = match kind {
        Kind::Wall => ["none", "low", "tall"],
        _ => ["false", "true", "true"],
    };
    let side = |name: &str| digit(entry, name, side_values);
    Family {
        kind,
        base: entry.base_state_id.number(),
        sides: Sides {
            north: side("north"),
            east: side("east"),
            south: side("south"),
            west: side("west"),
        },
        up: digit(entry, "up", ["false", "true", "true"]),
        in_wall: digit(entry, "in_wall", ["false", "true", "true"]),
    }
}

fn digit(entry: &BlockEntry, name: &str, texts: [&str; 3]) -> Digit {
    let Some(index) = entry.properties.index_of(name) else {
        return Digit::default();
    };
    let Some((property, later)) = entry
        .properties
        .0
        .get(index..)
        .and_then(|properties| properties.split_first())
    else {
        return Digit::default();
    };
    let stride = later
        .iter()
        .map(|property| u16::try_from(property.values.len()).unwrap_or(u16::MAX))
        .fold(1u16, u16::saturating_mul);
    let position = |text: &str| {
        property
            .values
            .iter()
            .position(|value| value.renders_to(text))
            .and_then(|position| u16::try_from(position).ok())
    };
    let values = texts.map(position);
    let (Some(first), Some(second), Some(third)) = (values[0], values[1], values[2]) else {
        return Digit::default();
    };
    Digit {
        stride: stride.max(1),
        count: u16::try_from(property.values.len()).unwrap_or(0),
        values: [first, second, third],
    }
}

pub(crate) fn traits_of(
    definitions: &BlockDefinitions,
    entry: &BlockEntry,
    state: BlockStateId,
    block: Id<Block>,
    tags: Option<&Tags<Block>>,
    sets: &TagSet,
) -> Traits {
    let mut bits = 0;
    let support = definitions.shape(definitions.support_shape(state));
    let collision = definitions.shape(definitions.state(state).collision_shape);
    for side in Direction::HORIZONTAL {
        let Some(bit) = horizontal_bit(side) else {
            continue;
        };
        if face_test::passes(support, side, FaceTest::Full) {
            bits |= FULL_FACE << bit;
        }
        if face_test::bottom_covers(collision, face_test::arm_area(side)) {
            bits |= COVERS_ARM << bit;
        }
    }
    if face_test::bottom_covers(collision, face_test::POST_AREA) {
        bits |= COVERS_POST;
    }
    if is_excepted_block(block)
        || in_tag(tags, sets.shulker_boxes, block)
        || in_tag(tags, sets.leaves, block)
    {
        bits |= EXCEPTED;
    }
    if in_tag(tags, sets.wall_post_override, block) {
        bits |= POST_OVERRIDE;
    }
    if in_tag(tags, sets.wooden_fences, block) {
        bits |= WOODEN_FENCE;
    } else if in_tag(tags, sets.fences, block) {
        bits |= OTHER_FENCE;
    }
    if in_tag(tags, sets.fence_gates, block) {
        let along_x = entry
            .value_of(state, "facing")
            .is_some_and(|facing| facing.renders_to("east") || facing.renders_to("west"));
        bits |= if along_x { GATE_ALONG_X } else { GATE_ALONG_Z };
    }
    if is_pane(block) {
        bits |= PANE;
    }
    if in_tag(tags, sets.bars, block) {
        bits |= BARS;
    }
    if in_tag(tags, sets.walls, block) {
        bits |= WALL;
    }
    Traits(bits)
}
