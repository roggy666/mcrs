use crate::WolfSoundVariantId;
use crate::entity_form::EntityForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct WolfSoundVariant(pub WolfSoundVariantId);

impl EntityForm for WolfSoundVariant {}
