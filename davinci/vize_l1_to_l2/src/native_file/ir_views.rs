use super::{NativeSfc, NativeSfcObservation};

impl<'a> NativeSfcObservation<'a> {
    pub fn l1ir(&self) -> &vize_l1::L1Ir<'a> {
        self.descriptor()
    }
}

impl<'o, 'a> NativeSfc<'o, 'a> {
    pub fn l2ir(&self) -> &'o vize_l2::L2Ir<'a> {
        self.file().file()
    }
}
