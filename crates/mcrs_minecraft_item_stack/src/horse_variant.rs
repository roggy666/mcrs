use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum HorseVariant {
    White,
    Creamy,
    Chestnut,
    Brown,
    Black,
    Gray,
    DarkBrown,
}

impl ListName for HorseVariant {
    const ALL: &'static [Self] = &[
        Self::White,
        Self::Creamy,
        Self::Chestnut,
        Self::Brown,
        Self::Black,
        Self::Gray,
        Self::DarkBrown,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Creamy => "creamy",
            Self::Chestnut => "chestnut",
            Self::Brown => "brown",
            Self::Black => "black",
            Self::Gray => "gray",
            Self::DarkBrown => "dark_brown",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::White => 0,
            Self::Creamy => 1,
            Self::Chestnut => 2,
            Self::Brown => 3,
            Self::Black => 4,
            Self::Gray => 5,
            Self::DarkBrown => 6,
        }
    }
}

impl EntityForm for HorseVariant {}
