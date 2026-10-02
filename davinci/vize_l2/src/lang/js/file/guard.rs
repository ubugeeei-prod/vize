//! An observer unwind cannot seal an interrupted admitted Program as complete.

use crate::file::build::Facts;
pub(super) struct ProgramWalkGuard<'f, 'a> {
    facts: &'f mut Facts<'a>,
    row: usize,
}

impl<'f, 'a> ProgramWalkGuard<'f, 'a> {
    pub(super) fn new(facts: &'f mut Facts<'a>) -> Option<Self> {
        // The actual checked unit constructor just appended this private row.
        let row = facts.units.len().checked_sub(1)?;
        facts.units.get(row)?;
        Some(Self { facts, row })
    }

    pub(super) fn facts(&mut self) -> &mut Facts<'a> {
        self.facts
    }

    /// Normal processing ends after the sole walk/row closure or an explicit
    /// invalid-profile refusal; that refusal already keeps the unit incomplete.
    pub(super) fn complete(self) {
        if let Some(unit) = self.facts.units.get_mut(self.row) {
            unit.complete_walk();
        }
    }
}

impl Drop for ProgramWalkGuard<'_, '_> {
    fn drop(&mut self) {
        if let Some(unit) = self.facts.units.get_mut(self.row) {
            // Inline Pending→Interrupted only: no allocation/callback/assertion.
            unit.interrupt_walk();
        }
    }
}
