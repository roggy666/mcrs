#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    Walker,
    Omnidirectional,
    Flier,
    Item,
    FallingBlock,
    PrimedTnt,
    ExperienceOrb,
    Projectile,
}

impl Category {
    #[must_use]
    pub const fn is_living(self) -> bool {
        matches!(
            self,
            Category::Walker | Category::Omnidirectional | Category::Flier
        )
    }
}
