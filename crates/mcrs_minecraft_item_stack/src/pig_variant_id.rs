use crate::NameTable;
use crate::variant_id::VariantId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PigVariantId(pub u16);

impl VariantId for PigVariantId {
    const TABLE: NameTable = NameTable::PigVariant;

    fn new(id: u16) -> Self {
        Self(id)
    }

    fn get(self) -> u16 {
        self.0
    }
}
