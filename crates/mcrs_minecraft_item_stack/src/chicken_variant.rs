use crate::ChickenVariantId;
use crate::entity_form::EntityForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct ChickenVariant(pub ChickenVariantId);

impl EntityForm for ChickenVariant {}
