#[cfg(feature = "bevy")]
use bevy_ecs::component::Component;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct Exposure {
    pub in_sun: bool,
    pub in_water: bool,
    pub in_rain: bool,
}

impl Exposure {
    #[must_use]
    pub const fn wet(&self) -> bool {
        self.in_water || self.in_rain
    }
}
