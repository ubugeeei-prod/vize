use super::NativeSfc;

impl<'o, 'a> NativeSfc<'o, 'a> {
    pub fn l2ir(&self) -> &'o vize_l2::L2Ir<'a> {
        self.file().l2ir()
    }
}
