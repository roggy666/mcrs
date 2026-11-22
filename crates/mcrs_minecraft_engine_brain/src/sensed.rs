use mcrs_minecraft_damage::Exposure;
use mcrs_minecraft_damage::Fire;
use mcrs_minecraft_damage::HitRecord;
use mcrs_minecraft_damage::Vitals;
use mcrs_minecraft_physics::EntityState;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Sensed<'a> {
    pub(crate) state: &'a EntityState,
    pub(crate) vitals: Option<&'a Vitals>,
    pub(crate) hits: Option<&'a HitRecord>,
    pub(crate) fire: Option<&'a Fire>,
    pub(crate) exposure: Option<&'a Exposure>,
}

impl Sensed<'_> {
    pub(crate) fn is_alive(&self) -> bool {
        self.vitals.is_none_or(Vitals::is_alive)
    }

    pub(crate) fn burning(&self) -> bool {
        self.fire.is_some_and(Fire::burning)
    }

    pub(crate) fn exposure(&self) -> Exposure {
        self.exposure.copied().unwrap_or_default()
    }
}
