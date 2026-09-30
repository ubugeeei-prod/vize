//! Strict selection accounting and whole-module parity failures.

use super::diff::divergence;
use super::shapes::{Shape, compile};
use super::tally::{Lane, Tally};
use crate::Sweep;
use vize_atelier_sfc::{SfcCompileResult, SfcDescriptor};

/// A normal script SFC returns template diagnostics on `Ok` when the template
/// is rejected before any backend runs. That is not a missing selection.
pub fn absorb_prebackend(
    descriptor: &SfcDescriptor<'_>,
    name: &str,
    shape: Shape,
    tally: &mut Tally,
    selected: &SfcCompileResult,
    lane: &Lane,
) -> bool {
    if *lane != Lane::Unrecorded
        || !selected
            .errors
            .iter()
            .any(|error| error.code.as_deref() == Some("TEMPLATE_ERROR"))
    {
        return false;
    }
    *tally
        .sfc_errors
        .entry("TEMPLATE_ERROR".to_owned())
        .or_default() += 1;
    if shape.is_dom() {
        let legacy =
            vize_atelier_dom::differential::with_legacy_lane(|| compile(descriptor, name, shape));
        if let Some(found) = divergence(selected, &legacy) {
            tally
                .divergences
                .push(format!("{name} [{}]: {found}", shape.id()));
        }
    }
    true
}

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
