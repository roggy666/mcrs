use crate::entity_form::EntityForm;
use crate::list_name::ListName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub enum TropicalFishPattern {
    Kob,
    Sunstreak,
    Snooper,
    Dasher,
    Brinely,
    Spotty,
    Flopper,
    Stripey,
    Glitter,
    Blockfish,
    Betty,
    Clayfish,
}

impl ListName for TropicalFishPattern {
    const ALL: &'static [Self] = &[
        Self::Kob,
        Self::Sunstreak,
        Self::Snooper,
        Self::Dasher,
        Self::Brinely,
        Self::Spotty,
        Self::Flopper,
        Self::Stripey,
        Self::Glitter,
        Self::Blockfish,
        Self::Betty,
        Self::Clayfish,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Kob => "kob",
            Self::Sunstreak => "sunstreak",
            Self::Snooper => "snooper",
            Self::Dasher => "dasher",
            Self::Brinely => "brinely",
            Self::Spotty => "spotty",
            Self::Flopper => "flopper",
            Self::Stripey => "stripey",
            Self::Glitter => "glitter",
            Self::Blockfish => "blockfish",
            Self::Betty => "betty",
            Self::Clayfish => "clayfish",
        }
    }

    fn id(self) -> i32 {
        match self {
            Self::Kob => 0,
            Self::Sunstreak => 256,
            Self::Snooper => 512,
            Self::Dasher => 768,
            Self::Brinely => 1024,
            Self::Spotty => 1280,
            Self::Flopper => 1,
            Self::Stripey => 257,
            Self::Glitter => 513,
            Self::Blockfish => 769,
            Self::Betty => 1025,
            Self::Clayfish => 1281,
        }
    }
}

impl EntityForm for TropicalFishPattern {}
