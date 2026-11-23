use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum LlamaVariant {
    Creamy,
    White,
    Brown,
    Gray,
}

impl ListName for LlamaVariant {
    const ALL: &'static [Self] = &[Self::Creamy, Self::White, Self::Brown, Self::Gray];

    fn name(self) -> &'static str {
        match self {
            Self::Creamy => "creamy",
            Self::White => "white",
            Self::Brown => "brown",
            Self::Gray => "gray",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::Creamy => 0,
            Self::White => 1,
            Self::Brown => 2,
            Self::Gray => 3,
        }
    }
}

impl EntityForm for LlamaVariant {}
