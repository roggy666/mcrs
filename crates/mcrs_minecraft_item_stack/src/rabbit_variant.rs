use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum RabbitVariant {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    Evil,
}

impl ListName for RabbitVariant {
    const ALL: &'static [Self] = &[
        Self::Brown,
        Self::White,
        Self::Black,
        Self::WhiteSplotched,
        Self::Gold,
        Self::Salt,
        Self::Evil,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Brown => "brown",
            Self::White => "white",
            Self::Black => "black",
            Self::WhiteSplotched => "white_splotched",
            Self::Gold => "gold",
            Self::Salt => "salt",
            Self::Evil => "evil",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::Brown => 0,
            Self::White => 1,
            Self::Black => 2,
            Self::WhiteSplotched => 3,
            Self::Gold => 4,
            Self::Salt => 5,
            Self::Evil => 99,
        }
    }
}

impl EntityForm for RabbitVariant {}
