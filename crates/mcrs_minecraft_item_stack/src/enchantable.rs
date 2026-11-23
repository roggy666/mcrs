use crate::entity_form::EntityForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct Enchantable(pub i32);

impl Enchantable {
    #[must_use]
    pub const fn new(value: i32) -> Option<Self> {
        if value >= 1 { Some(Self(value)) } else { None }
    }
}

impl EntityForm for Enchantable {}
