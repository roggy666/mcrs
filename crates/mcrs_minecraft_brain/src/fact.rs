use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fact {
    pub mob: EntityId,
    pub key: EntityUuid,
    pub kind: FactKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FactKind {
    Hurt,
    PathEnded,
    RiderChanged,
    DisabledChanged,
    IdleReset,
    LoveOrBaby,
    Resumed,
    Dropped,
    Alerted,
    TargetLost,
    Tamed,
}
