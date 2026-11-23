#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Filterable<T> {
    pub raw: T,
    pub filtered: Option<T>,
}

impl<T> Filterable<T> {
    #[must_use]
    pub const fn new(raw: T) -> Self {
        Self {
            raw,
            filtered: None,
        }
    }
}
