use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum AxolotlVariant {
    Lucy,
    Wild,
    Gold,
    Cyan,
    Blue,
}

impl ListName for AxolotlVariant {
    const ALL: &'static [Self] = &[Self::Lucy, Self::Wild, Self::Gold, Self::Cyan, Self::Blue];

    fn name(self) -> &'static str {
        match self {
            Self::Lucy => "lucy",
            Self::Wild => "wild",
            Self::Gold => "gold",
            Self::Cyan => "cyan",
            Self::Blue => "blue",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::Lucy => 0,
            Self::Wild => 1,
            Self::Gold => 2,
            Self::Cyan => 3,
            Self::Blue => 4,
        }
    }
}

impl EntityForm for AxolotlVariant {}
