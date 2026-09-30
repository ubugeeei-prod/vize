//! The remarks observer: collects pass remarks for `vize dump --remarks`.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::{PassEvent, PassObserver, Remark};

/// One collected remark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemarkRecord {
    /// The pass that emitted it.
    pub pass: &'static str,
    /// Stable remark id.
    pub id: Box<str>,
    /// Human message.
    pub message: Box<str>,
}

/// Collects every remark a pipeline emits.
#[derive(Debug, Clone, Default)]
pub struct RemarkCollector {
    records: Vec<RemarkRecord>,
}

impl RemarkCollector {
    /// The remarks collected so far.
    #[must_use]
    pub fn records(&self) -> &[RemarkRecord] {
        &self.records
    }
}

impl PassObserver for RemarkCollector {
    fn on_remark(&mut self, event: &PassEvent<'_>, remark: &Remark<'_>) {
        let _ = (event, remark);
        todo!()
    }
}
