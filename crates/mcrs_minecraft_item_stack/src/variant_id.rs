use crate::NameTable;

pub(crate) trait VariantId: Copy {
    const TABLE: NameTable;

    fn new(id: u16) -> Self;

    fn get(self) -> u16;
}
