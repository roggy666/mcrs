use crate::PigSoundVariantId;
use crate::entity_form::EntityForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct PigSoundVariant(pub PigSoundVariantId);

impl EntityForm for PigSoundVariant {}
