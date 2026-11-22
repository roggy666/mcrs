use core::fmt;

use mcrs_minecraft_block::block_keys::Block;
use mcrs_minecraft_block::definition::BlockDefinitions;
use mcrs_minecraft_core::Direction;
use mcrs_minecraft_core::direction::Axis;
use mcrs_minecraft_registries::Registries;
use mcrs_minecraft_resource::{ResourceLocation, ResourceLocationBuf, TagKey};
use mcrs_voxel::{BlockPos, BlockStateId};

use crate::block_connections_bake::{TagSet, family_of, kind_of, traits_of};

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::resource::Resource))]
pub struct BlockConnections {
    traits: Box<[Traits]>,
    subjects: Box<[Option<u16>]>,
    families: Vec<Family>,
    state_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionIssue {
    pub tag: ResourceLocationBuf,
}

impl fmt::Display for ConnectionIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the connection rules read the unknown block tag {}",
            self.tag
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Traits(pub(crate) u32);

pub(crate) const FULL_FACE: u32 = 1;
pub(crate) const COVERS_ARM: u32 = 1 << 4;
pub(crate) const COVERS_POST: u32 = 1 << 8;
pub(crate) const EXCEPTED: u32 = 1 << 9;
pub(crate) const POST_OVERRIDE: u32 = 1 << 10;
pub(crate) const WOODEN_FENCE: u32 = 1 << 11;
pub(crate) const OTHER_FENCE: u32 = 1 << 12;
pub(crate) const GATE_ALONG_X: u32 = 1 << 13;
pub(crate) const GATE_ALONG_Z: u32 = 1 << 14;
pub(crate) const PANE: u32 = 1 << 15;
pub(crate) const BARS: u32 = 1 << 16;
pub(crate) const WALL: u32 = 1 << 17;

impl Traits {
    fn has(self, bit: u32) -> bool {
        self.0 & bit != 0
    }

    fn full_face(self, side: Direction) -> bool {
        horizontal_bit(side).is_some_and(|bit| self.has(FULL_FACE << bit))
    }

    fn covers_arm(self, side: Direction) -> bool {
        horizontal_bit(side).is_some_and(|bit| self.has(COVERS_ARM << bit))
    }

    fn gate_across(self, side: Direction) -> bool {
        match side.axis() {
            Axis::X => self.has(GATE_ALONG_Z),
            Axis::Z => self.has(GATE_ALONG_X),
            Axis::Y => false,
        }
    }

    fn is_fence_or_gate(self) -> bool {
        self.has(WOODEN_FENCE | OTHER_FENCE | GATE_ALONG_X | GATE_ALONG_Z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Fence,
    PaneOrBars,
    Wall,
    Gate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    None,
    Low,
    Tall,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Digit {
    pub(crate) stride: u16,
    pub(crate) count: u16,
    pub(crate) values: [u16; 3],
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Sides {
    pub(crate) north: Digit,
    pub(crate) east: Digit,
    pub(crate) south: Digit,
    pub(crate) west: Digit,
}

impl Sides {
    fn of(self, side: Direction) -> Digit {
        match side {
            Direction::North => self.north,
            Direction::East => self.east,
            Direction::South => self.south,
            Direction::West => self.west,
            Direction::Down | Direction::Up => Digit::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Family {
    pub(crate) kind: Kind,
    pub(crate) base: u16,
    pub(crate) sides: Sides,
    pub(crate) up: Digit,
    pub(crate) in_wall: Digit,
}

impl Family {
    fn read(&self, state: BlockStateId, digit: Digit) -> u16 {
        if digit.count == 0 {
            return 0;
        }
        state.number().wrapping_sub(self.base) / digit.stride % digit.count
    }

    fn write(&self, state: BlockStateId, digit: Digit, value: usize, len: usize) -> BlockStateId {
        let Some(target) = digit.values.get(value).copied() else {
            return state;
        };
        if digit.count == 0 {
            return state;
        }
        let current = self.read(state, digit);
        let number = state
            .number()
            .wrapping_sub(current.wrapping_mul(digit.stride))
            .wrapping_add(target.wrapping_mul(digit.stride));
        BlockStateId::checked(number, len).unwrap_or(state)
    }

    fn side(&self, state: BlockStateId, side: Direction) -> Side {
        let digit = self.sides.of(side);
        let value = self.read(state, digit);
        if value == digit.values[2] && self.kind == Kind::Wall {
            Side::Tall
        } else if value == digit.values[1] {
            Side::Low
        } else {
            Side::None
        }
    }

    fn with_side(
        &self,
        state: BlockStateId,
        side: Direction,
        value: Side,
        len: usize,
    ) -> BlockStateId {
        let index = match value {
            Side::None => 0,
            Side::Low => 1,
            Side::Tall => 2,
        };
        self.write(state, self.sides.of(side), index, len)
    }
}

pub(crate) fn horizontal_bit(direction: Direction) -> Option<u32> {
    match direction {
        Direction::North => Some(0),
        Direction::East => Some(1),
        Direction::South => Some(2),
        Direction::West => Some(3),
        Direction::Down | Direction::Up => None,
    }
}

impl BlockConnections {
    pub fn bake(
        definitions: &BlockDefinitions,
        registries: &Registries,
    ) -> (Self, Vec<ConnectionIssue>) {
        let tags = registries.tags::<Block>();
        let mut issues = Vec::new();
        let mut tag = |name: &'static str| {
            let Ok(location) = ResourceLocation::read(name) else {
                return None;
            };
            let key: TagKey<'static, Block> = TagKey::new(location);
            let found = tags.as_ref().and_then(|tags| tags.id(key));
            if found.is_none() {
                issues.push(ConnectionIssue {
                    tag: location.to_buf(),
                });
            }
            found
        };
        let sets = TagSet {
            wooden_fences: tag("minecraft:wooden_fences"),
            fences: tag("minecraft:fences"),
            fence_gates: tag("minecraft:fence_gates"),
            walls: tag("minecraft:walls"),
            bars: tag("minecraft:bars"),
            shulker_boxes: tag("minecraft:shulker_boxes"),
            leaves: tag("minecraft:leaves"),
            wall_post_override: tag("minecraft:wall_post_override"),
        };
        let state_count = definitions.state_count();
        let mut traits = vec![Traits::default(); state_count].into_boxed_slice();
        let mut subjects = vec![None; state_count].into_boxed_slice();
        let mut families = Vec::new();
        for entry in definitions.blocks() {
            let Some(block) = definitions.id_of(entry.identifier.as_str()) else {
                continue;
            };
            let kind = kind_of(block, tags.as_ref(), &sets);
            let family = kind.map(|kind| family_of(entry, kind));
            let family_index = family.and_then(|family| {
                let index = u16::try_from(families.len()).ok()?;
                families.push(family);
                Some(index)
            });
            for offset in 0..entry.state_count {
                let Some(state) = entry.state_at(offset) else {
                    continue;
                };
                let Some(slot) = traits.get_mut(state.index()) else {
                    continue;
                };
                *slot = traits_of(definitions, entry, state, block, tags.as_ref(), &sets);
                if let Some(subject) = subjects.get_mut(state.index()) {
                    *subject = family_index;
                }
            }
        }
        let connections = BlockConnections {
            traits,
            subjects,
            families,
            state_count,
        };
        (connections, issues)
    }

    #[must_use]
    pub fn after_update(
        &self,
        state: BlockStateId,
        at: BlockPos,
        from: Direction,
        neighbour: BlockStateId,
        mut read: impl FnMut(BlockPos) -> BlockStateId,
    ) -> BlockStateId {
        let Some(family) = self.family(state) else {
            return state;
        };
        let me = self.traits_of(state);
        let them = self.traits_of(neighbour);
        match family.kind {
            Kind::Gate => {
                if from.is_vertical() || !me.gate_across(from) {
                    return state;
                }
                let other = self.traits_of(read(at + from.opposite().normal()));
                let in_wall = them.has(WALL) || other.has(WALL);
                family.write(
                    state,
                    family.in_wall,
                    usize::from(in_wall),
                    self.state_count,
                )
            }
            Kind::Fence | Kind::PaneOrBars => {
                if from.is_vertical() {
                    return state;
                }
                let value = if connects(family.kind, me, from, them) {
                    Side::Low
                } else {
                    Side::None
                };
                family.with_side(state, from, value, self.state_count)
            }
            Kind::Wall => {
                let above = match from {
                    Direction::Up => them,
                    Direction::Down => return state,
                    _ => self.traits_of(read(at + Direction::Up.normal())),
                };
                let mut next = state;
                for side in Direction::HORIZONTAL {
                    let connected = if side == from {
                        connects(Kind::Wall, me, side, them)
                    } else {
                        family.side(state, side) != Side::None
                    };
                    let value = match (connected, above.covers_arm(side)) {
                        (false, _) => Side::None,
                        (true, true) => Side::Tall,
                        (true, false) => Side::Low,
                    };
                    next = family.with_side(next, side, value, self.state_count);
                }
                let sides = Direction::HORIZONTAL.map(|side| family.side(next, side));
                let up = post(sides, above);
                family.write(next, family.up, usize::from(up), self.state_count)
            }
        }
    }

    fn family(&self, state: BlockStateId) -> Option<&Family> {
        let index = self.subjects.get(state.index()).copied().flatten()?;
        self.families.get(usize::from(index))
    }

    fn traits_of(&self, state: BlockStateId) -> Traits {
        self.traits.get(state.index()).copied().unwrap_or_default()
    }
}

fn connects(kind: Kind, me: Traits, side: Direction, them: Traits) -> bool {
    let face = them.full_face(side.opposite()) && !them.has(EXCEPTED) && !them.is_fence_or_gate();
    match kind {
        Kind::Fence => {
            (me.has(WOODEN_FENCE) && them.has(WOODEN_FENCE))
                || (me.has(OTHER_FENCE) && them.has(OTHER_FENCE))
                || them.gate_across(side)
                || face
        }
        Kind::PaneOrBars => them.has(PANE | BARS | WALL) || face,
        Kind::Wall => them.has(WALL | PANE | BARS) || them.gate_across(side) || face,
        Kind::Gate => false,
    }
}

fn post(sides: [Side; 4], above: Traits) -> bool {
    if above.has(POST_OVERRIDE) {
        return true;
    }
    let [north, east, south, west] = sides;
    let connected = |side: Side| side != Side::None;
    let along_x = connected(east) && connected(west);
    let along_z = connected(north) && connected(south);
    let line = (along_x && !connected(north) && !connected(south))
        || (along_z && !connected(east) && !connected(west))
        || (along_x && along_z);
    if !line {
        return true;
    }
    let tall_pair =
        (east == Side::Tall && west == Side::Tall) || (north == Side::Tall && south == Side::Tall);
    above.has(COVERS_POST) && !tall_pair
}
