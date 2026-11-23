use crate::NameTable;
use crate::variant_id::VariantId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CatSoundVariantId(pub u16);

impl VariantId for CatSoundVariantId {
    const TABLE: NameTable = NameTable::CatSoundVariant;

    fn new(id: u16) -> Self {
        Self(id)
    }

    fn get(self) -> u16 {
        self.0
    }
}
