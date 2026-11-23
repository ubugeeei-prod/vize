use crate::{Lowered, lower};
use vize_l0::Allocator;

impl<'a> Lowered<'a> {
    #[must_use]
    pub fn l3ir(&self) -> &vize_l3::L3Ir<'a> {
        &self.program
    }
}

/// Lower the root from a complete native L2 file.
#[must_use]
pub fn lower_ir<'a>(allocator: &'a Allocator, file: &vize_l2::L2Ir<'_>) -> Lowered<'a> {
    lower(allocator, file.root_region())
}
