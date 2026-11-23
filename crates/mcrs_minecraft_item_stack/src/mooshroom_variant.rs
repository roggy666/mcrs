use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum MooshroomVariant {
    Red,
    Brown,
}

impl ListName for MooshroomVariant {
    const ALL: &'static [Self] = &[Self::Red, Self::Brown];

    fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Brown => "brown",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::Red => 0,
            Self::Brown => 1,
        }
    }
}

impl EntityForm for MooshroomVariant {}
