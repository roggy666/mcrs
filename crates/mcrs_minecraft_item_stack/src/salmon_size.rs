use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum SalmonSize {
    Small,
    Medium,
    Large,
}

impl ListName for SalmonSize {
    const ALL: &'static [Self] = &[Self::Small, Self::Medium, Self::Large];

    fn name(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
        }
    }
}

impl EntityForm for SalmonSize {}
