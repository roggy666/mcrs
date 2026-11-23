use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum FoxVariant {
    Red,
    Snow,
}

impl ListName for FoxVariant {
    const ALL: &'static [Self] = &[Self::Red, Self::Snow];

    fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Snow => "snow",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::Red => 0,
            Self::Snow => 1,
        }
    }
}

impl EntityForm for FoxVariant {}
