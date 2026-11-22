use bevy_ecs::message::Message;
use mcrs_minecraft_brain::Proposal;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTarget {
    Mob(EntityId, EntityUuid),
    Player(EntityId, EntityUuid),
}

impl HitTarget {
    #[must_use]
    pub const fn key(&self) -> EntityUuid {
        match self {
            Self::Mob(_, key) | Self::Player(_, key) => *key,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeleeHit {
    pub attacker: EntityId,
    pub attacker_key: EntityUuid,
    pub target: HitTarget,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangedShot {
    pub attacker: EntityId,
    pub target: EntityId,
    pub power: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Proposed {
    pub proposer: EntityId,
    pub key: EntityUuid,
    pub issue_tick: u64,
    pub proposal: Proposal,
}

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct MobHit {
    pub attacker: EntityId,
    pub target: EntityId,
    pub amount: f32,
    pub damage_type: u32,
}

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct RangedAttack {
    pub attacker: EntityId,
    pub target: EntityId,
    pub power: f32,
}
