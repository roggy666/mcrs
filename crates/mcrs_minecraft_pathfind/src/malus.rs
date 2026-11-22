use mcrs_minecraft_path_type::PathType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MalusTable([f32; PathType::COUNT]);

impl Default for MalusTable {
    fn default() -> Self {
        let mut table = [0.0; PathType::COUNT];
        for (index, malus) in table.iter_mut().enumerate() {
            if let Some(kind) = PathType::from_number(index as u8) {
                *malus = kind.default_malus();
            }
        }
        Self(table)
    }
}

impl MalusTable {
    #[must_use]
    pub fn get(&self, kind: PathType) -> f32 {
        self.0
            .get(usize::from(kind.number()))
            .copied()
            .unwrap_or(-1.0)
    }

    pub fn set(&mut self, kind: PathType, malus: f32) {
        if let Some(slot) = self.0.get_mut(usize::from(kind.number())) {
            *slot = malus;
        }
    }

    #[must_use]
    pub fn with(mut self, kind: PathType, malus: f32) -> Self {
        self.set(kind, malus);
        self
    }
}
