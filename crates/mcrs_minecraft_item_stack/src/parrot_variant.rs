use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum ParrotVariant {
    RedBlue,
    Blue,
    Green,
    YellowBlue,
    Gray,
}

impl ListName for ParrotVariant {
    const ALL: &'static [Self] = &[
        Self::RedBlue,
        Self::Blue,
        Self::Green,
        Self::YellowBlue,
        Self::Gray,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::RedBlue => "red_blue",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::YellowBlue => "yellow_blue",
            Self::Gray => "gray",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::RedBlue => 0,
            Self::Blue => 1,
            Self::Green => 2,
            Self::YellowBlue => 3,
            Self::Gray => 4,
        }
    }
}

impl EntityForm for ParrotVariant {}
