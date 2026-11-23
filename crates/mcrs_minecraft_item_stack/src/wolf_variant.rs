use crate::WolfVariantId;
use crate::entity_form::EntityForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct WolfVariant(pub WolfVariantId);

impl EntityForm for WolfVariant {}
