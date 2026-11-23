use crate::DyeColor;
use crate::entity_form::EntityForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct TropicalFishPatternColor(pub DyeColor);

impl EntityForm for TropicalFishPatternColor {}
