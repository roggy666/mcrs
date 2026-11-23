use crate::NameTable;
use crate::variant_id::VariantId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WolfSoundVariantId(pub u16);

impl VariantId for WolfSoundVariantId {
    const TABLE: NameTable = NameTable::WolfSoundVariant;

    fn new(id: u16) -> Self {
        Self(id)
    }

    fn get(self) -> u16 {
        self.0
    }
}
