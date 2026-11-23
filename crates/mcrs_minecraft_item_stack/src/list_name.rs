pub(crate) trait ListName: Copy + PartialEq + 'static {
    const ALL: &'static [Self];

    fn name(self) -> &'static str;

    fn id(self) -> i32;

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|entry| entry.name() == name)
    }

    fn from_id(id: i32) -> Option<Self> {
        Self::ALL.iter().copied().find(|entry| entry.id() == id)
    }
}
