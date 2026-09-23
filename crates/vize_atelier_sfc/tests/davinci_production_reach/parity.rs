//! Strict selection accounting and whole-module parity failures.

use crate::Sweep;

pub fn parity_failures(sweep: &Sweep) -> Vec<String> {
    let mut failures = Vec::new();
    for (shape, tally) in sweep.tallies.values() {
        failures.extend(tally.violations.iter().cloned());
        failures.extend(tally.divergences.iter().take(10).cloned());
        if shape.is_dom() && tally.unrecorded != 0 {
            failures.push(format!(
                "{}: {} DOM template compiles recorded no selection counter: {:?}",
                shape.id(),
                tally.unrecorded,
                tally.unrecorded_samples
            ));
        }
    }
    failures
}
