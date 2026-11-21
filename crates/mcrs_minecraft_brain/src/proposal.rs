use mcrs_minecraft_entity_key::EntityId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Proposal {
    Breed { partner: EntityId },
    Alert { target: EntityId, reach: f32 },
    CaravanJoin { anchor: EntityId },
    CaravanLeave,
}

impl Proposal {
    #[must_use]
    pub const fn other(&self) -> Option<EntityId> {
        match self {
            Self::Breed { partner } => Some(*partner),
            Self::Alert { target, .. } => Some(*target),
            Self::CaravanJoin { anchor } => Some(*anchor),
            Self::CaravanLeave => None,
        }
    }
}
