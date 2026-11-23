use crate::NameTable;
use crate::variant_id::VariantId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CowVariantId(pub u16);

impl VariantId for CowVariantId {
    const TABLE: NameTable = NameTable::CowVariant;

    fn new(id: u16) -> Self {
        Self(id)
    }

    fn get(self) -> u16 {
        self.0
    }
}
