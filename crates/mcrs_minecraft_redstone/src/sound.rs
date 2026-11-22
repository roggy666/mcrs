use mcrs_minecraft_resource::Id;
use mcrs_minecraft_simulation::{
    BlockBehaviour, BlockFacts, Cx, Kinds, PitchRange, SimulationVolume,
};
use mcrs_minecraft_sound::sound_event_keys::SoundEvent;
use mcrs_voxel::BlockPos;

use crate::component::Kind;
use crate::component::PlateKind;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Click {
    pub sound: Id<SoundEvent>,
    pub volume: f32,
    pub pitch: PitchRange,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SoundSet {
    #[default]
    Other,
    Wood,
    CherryWood,
    BambooWood,
    NetherWood,
    Stone,
    Metal,
    Copper,
}

const CLICK_VOLUME: f32 = 0.3;
const TRIPWIRE_VOLUME: f32 = 0.4;
const DOOR_VOLUME: f32 = 0.9;
const LOUD: f32 = 1.0;
const PISTON_VOLUME: f32 = 0.5;
const CREAK: PitchRange = PitchRange::between(0.9, 1.0);
const HIGH_CREAK: PitchRange = PitchRange::between(0.99, 1.1);

const fn click(sound: SoundEvent, volume: f32, pitch: f32) -> Click {
    Click {
        sound: sound.id(),
        volume,
        pitch: PitchRange::fixed(pitch),
    }
}

const fn creak(sound: SoundEvent, volume: f32, pitch: PitchRange) -> Click {
    Click {
        sound: sound.id(),
        volume,
        pitch,
    }
}

#[must_use]
pub fn lever(on: bool) -> Click {
    click(
        SoundEvent::BlockLeverClick,
        CLICK_VOLUME,
        if on { 0.6 } else { 0.5 },
    )
}

#[must_use]
pub fn button(set: SoundSet, on: bool) -> Click {
    use SoundEvent::*;
    let sound = match (set, on) {
        (SoundSet::CherryWood, true) => BlockCherryWoodButtonClickOn,
        (SoundSet::CherryWood, false) => BlockCherryWoodButtonClickOff,
        (SoundSet::BambooWood, true) => BlockBambooWoodButtonClickOn,
        (SoundSet::BambooWood, false) => BlockBambooWoodButtonClickOff,
        (SoundSet::NetherWood, true) => BlockNetherWoodButtonClickOn,
        (SoundSet::NetherWood, false) => BlockNetherWoodButtonClickOff,
        (SoundSet::Wood, true) => BlockWoodenButtonClickOn,
        (SoundSet::Wood, false) => BlockWoodenButtonClickOff,
        (_, true) => BlockStoneButtonClickOn,
        (_, false) => BlockStoneButtonClickOff,
    };
    click(sound, CLICK_VOLUME, if on { 0.6 } else { 0.5 })
}

#[must_use]
pub fn plate(kind: PlateKind, set: SoundSet, on: bool) -> Click {
    use SoundEvent::*;
    let wooden = matches!(kind, PlateKind::Wooden);
    let sound = match (set, wooden, on) {
        (SoundSet::CherryWood, true, true) => BlockCherryWoodPressurePlateClickOn,
        (SoundSet::CherryWood, true, false) => BlockCherryWoodPressurePlateClickOff,
        (SoundSet::BambooWood, true, true) => BlockBambooWoodPressurePlateClickOn,
        (SoundSet::BambooWood, true, false) => BlockBambooWoodPressurePlateClickOff,
        (SoundSet::NetherWood, true, true) => BlockNetherWoodPressurePlateClickOn,
        (SoundSet::NetherWood, true, false) => BlockNetherWoodPressurePlateClickOff,
        (_, true, true) => BlockWoodenPressurePlateClickOn,
        (_, true, false) => BlockWoodenPressurePlateClickOff,
        (_, false, true) => BlockStonePressurePlateClickOn,
        (_, false, false) => BlockStonePressurePlateClickOff,
    };
    let pitch = match (wooden, on) {
        (true, true) => 0.8,
        (true, false) => 0.7,
        (false, true) => 0.6,
        (false, false) => 0.5,
    };
    click(sound, CLICK_VOLUME, pitch)
}

#[must_use]
pub fn door(wooden: bool, set: SoundSet, open: bool) -> Click {
    use SoundEvent::*;
    match (wooden, set, open) {
        (true, SoundSet::CherryWood, true) => creak(BlockCherryWoodDoorOpen, DOOR_VOLUME, CREAK),
        (true, SoundSet::CherryWood, false) => creak(BlockCherryWoodDoorClose, DOOR_VOLUME, CREAK),
        (true, SoundSet::BambooWood, true) => creak(BlockBambooWoodDoorOpen, DOOR_VOLUME, CREAK),
        (true, SoundSet::BambooWood, false) => creak(BlockBambooWoodDoorClose, DOOR_VOLUME, CREAK),
        (true, SoundSet::NetherWood, true) => creak(BlockNetherWoodDoorOpen, LOUD, CREAK),
        (true, SoundSet::NetherWood, false) => creak(BlockNetherWoodDoorClose, LOUD, CREAK),
        (true, _, true) => creak(BlockWoodenDoorOpen, DOOR_VOLUME, CREAK),
        (true, _, false) => creak(BlockWoodenDoorClose, DOOR_VOLUME, CREAK),
        (false, SoundSet::Copper, true) => creak(BlockCopperDoorOpen, LOUD, CREAK),
        (false, SoundSet::Copper, false) => creak(BlockCopperDoorClose, LOUD, CREAK),
        (false, _, true) => creak(BlockIronDoorOpen, DOOR_VOLUME, CREAK),
        (false, _, false) => creak(BlockIronDoorClose, DOOR_VOLUME, CREAK),
    }
}

#[must_use]
pub fn trapdoor(wooden: bool, set: SoundSet, open: bool) -> Click {
    use SoundEvent::*;
    match (wooden, set, open) {
        (true, SoundSet::CherryWood, true) => creak(BlockCherryWoodTrapdoorOpen, LOUD, CREAK),
        (true, SoundSet::CherryWood, false) => creak(BlockCherryWoodTrapdoorClose, LOUD, CREAK),
        (true, SoundSet::BambooWood, true) => creak(BlockBambooWoodTrapdoorOpen, LOUD, CREAK),
        (true, SoundSet::BambooWood, false) => creak(BlockBambooWoodTrapdoorClose, LOUD, CREAK),
        (true, SoundSet::NetherWood, true) => creak(BlockNetherWoodTrapdoorOpen, LOUD, CREAK),
        (true, SoundSet::NetherWood, false) => creak(BlockNetherWoodTrapdoorClose, LOUD, CREAK),
        (true, _, true) => creak(BlockWoodenTrapdoorOpen, DOOR_VOLUME, CREAK),
        (true, _, false) => creak(BlockWoodenTrapdoorClose, DOOR_VOLUME, CREAK),
        (false, SoundSet::Copper, true) => creak(BlockCopperTrapdoorOpen, LOUD, CREAK),
        (false, SoundSet::Copper, false) => creak(BlockCopperTrapdoorClose, LOUD, CREAK),
        (false, _, true) => creak(BlockIronTrapdoorOpen, DOOR_VOLUME, CREAK),
        (false, _, false) => creak(BlockIronTrapdoorClose, DOOR_VOLUME, CREAK),
    }
}

#[must_use]
pub fn fence_gate(set: SoundSet, open: bool) -> Click {
    use SoundEvent::*;
    match (set, open) {
        (SoundSet::CherryWood, true) => {
            creak(BlockCherryWoodFenceGateOpen, DOOR_VOLUME, HIGH_CREAK)
        }
        (SoundSet::CherryWood, false) => {
            creak(BlockCherryWoodFenceGateClose, DOOR_VOLUME, HIGH_CREAK)
        }
        (SoundSet::BambooWood, true) => {
            creak(BlockBambooWoodFenceGateOpen, DOOR_VOLUME, HIGH_CREAK)
        }
        (SoundSet::BambooWood, false) => {
            creak(BlockBambooWoodFenceGateClose, DOOR_VOLUME, HIGH_CREAK)
        }
        (SoundSet::NetherWood, true) => creak(BlockNetherWoodFenceGateOpen, LOUD, CREAK),
        (SoundSet::NetherWood, false) => creak(BlockNetherWoodFenceGateClose, LOUD, CREAK),
        (_, true) => creak(BlockFenceGateOpen, DOOR_VOLUME, CREAK),
        (_, false) => creak(BlockFenceGateClose, DOOR_VOLUME, CREAK),
    }
}

#[must_use]
pub fn piston(extend: bool) -> Click {
    if extend {
        creak(
            SoundEvent::BlockPistonExtend,
            PISTON_VOLUME,
            PitchRange::between(0.6, 0.85),
        )
    } else {
        creak(
            SoundEvent::BlockPistonContract,
            PISTON_VOLUME,
            PitchRange::between(0.6, 0.8),
        )
    }
}

#[must_use]
pub fn torch_burnout() -> Click {
    creak(
        SoundEvent::BlockRedstoneTorchBurnout,
        PISTON_VOLUME,
        PitchRange::between(1.8, 3.4),
    )
}

#[must_use]
pub fn comparator(subtract: bool) -> Click {
    click(
        SoundEvent::BlockComparatorClick,
        CLICK_VOLUME,
        if subtract { 0.55 } else { 0.5 },
    )
}

#[must_use]
pub fn tripwire(on: bool) -> Click {
    click(
        if on {
            SoundEvent::BlockTripwireClickOn
        } else {
            SoundEvent::BlockTripwireClickOff
        },
        TRIPWIRE_VOLUME,
        if on { 0.6 } else { 0.5 },
    )
}

#[must_use]
pub fn tripwire_attach() -> Click {
    click(SoundEvent::BlockTripwireAttach, TRIPWIRE_VOLUME, 0.7)
}

#[must_use]
pub fn tripwire_detach() -> Click {
    creak(
        SoundEvent::BlockTripwireDetach,
        TRIPWIRE_VOLUME,
        PitchRange::between(12.0 / 11.0, 4.0 / 3.0),
    )
}

#[must_use]
pub fn copper_bulb(on: bool) -> Click {
    click(
        if on {
            SoundEvent::BlockCopperBulbTurnOn
        } else {
            SoundEvent::BlockCopperBulbTurnOff
        },
        LOUD,
        if on { 1.0 } else { 0.75 },
    )
}

#[must_use]
pub fn open_sound(kind: Kind, set: SoundSet, open: bool) -> Option<Click> {
    match kind {
        Kind::Door { wooden } => Some(door(wooden, set, open)),
        Kind::Trapdoor { wooden } => Some(trapdoor(wooden, set, open)),
        Kind::FenceGate => Some(fence_gate(set, open)),
        _ => None,
    }
}

pub(crate) fn play<K, V, F, B>(cx: &mut Cx<'_, K, V, F, B>, pos: BlockPos, click: Click)
where
    K: Kinds<Sound = Id<SoundEvent>>,
    V: SimulationVolume,
    F: BlockFacts<K>,
    B: BlockBehaviour<K>,
{
    cx.play_sound(pos, click.sound, click.volume, click.pitch);
}
