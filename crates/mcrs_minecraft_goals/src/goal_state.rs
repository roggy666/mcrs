use mcrs_minecraft_brain::EntityRef;
use mcrs_voxel::BlockPos;

use crate::Gate;
use crate::LookAroundGoal;
use crate::LookAtPlayerGoal;
use crate::StrollGoal;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GoalState {
    #[default]
    None,
    Breed(BreedState),
    Tempt(TemptState),
    Follow(FollowState),
    Eat(EatState),
    Stroll(StrollGoal),
    LookAtPlayer(LookAtPlayerGoal),
    LookAround(LookAroundGoal),
    Melee(MeleeState),
    Leap(Gate),
    Avoid(AvoidState),
    Ranged(RangedState),
    FollowOwner(FollowOwnerState),
    Beg(BegState),
    Target(TargetState),
    HurtBy(HurtByState),
    Owner(OwnerState),
    Anger(AngerState),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BreedState {
    pub partner: EntityRef,
    pub count: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TemptState {
    pub player: EntityRef,
    pub calm_until: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FollowState {
    pub adult: EntityRef,
    pub timer: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EatState {
    pub gate: Gate,
    pub timer: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MeleeState {
    pub last_check: u32,
    pub timer: u8,
    pub point: Option<BlockPos>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AvoidState {
    pub entity: EntityRef,
    pub destination: BlockPos,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RangedState {
    pub target: EntityRef,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FollowOwnerState {
    pub timer: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BegState {
    pub player: EntityRef,
    pub time: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TargetState {
    pub gate: Gate,
    pub unseen: u8,
    pub kept: EntityRef,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HurtByState {
    pub kept_tick: u32,
    pub unseen: u8,
    pub goal_target: EntityRef,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct OwnerState {
    pub kept_tick: u32,
    pub unseen: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AngerState {
    pub kept_tick: u32,
}

impl Default for AvoidState {
    fn default() -> Self {
        Self {
            entity: EntityRef::NONE,
            destination: BlockPos::new(0, 0, 0),
        }
    }
}
