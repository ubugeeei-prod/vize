//! Strict selection accounting and whole-module parity failures.

use super::diff::divergence;
use super::shapes::{Shape, compile};
use super::tally::{Lane, Tally};
use crate::Sweep;
use vize_atelier_sfc::{SfcCompileResult, SfcDescriptor};

/// A script-bearing SFC can return template diagnostics in `Ok` before any
/// backend runs. Record that rejection without inventing a selection, and
/// retain whole-module parity with the forced legacy path.
pub fn record_prebackend_template_error(
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
        if let Some(divergence) = divergence(selected, &legacy) {
            tally
                .divergences
                .push(format!("{name} [{}]: {divergence}", shape.id()));
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
