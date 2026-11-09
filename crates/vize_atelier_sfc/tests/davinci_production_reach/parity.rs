//! Strict selection accounting and whole-module parity failures.

use super::tally::{EMIT_REFUSED, Lane, Tally, classify};
use crate::Sweep;
use vize_atelier_sfc::dom_differential::diff::divergence;
use vize_atelier_sfc::dom_differential::shapes::{Shape, compile};
use vize_atelier_sfc::{SfcCompileResult, SfcDescriptor};
use vize_l0::profiler::global_profiler;

/// An SFC can return template diagnostics on `Ok` after its parser rejected the
/// template before backend selection. Keep that rejection and its original
/// diagnostics outside the successful-template denominator.
pub fn absorb_prebackend(
    descriptor: &SfcDescriptor<'_>,
    name: &str,
    shape: Shape,
    tally: &mut Tally,
    selected: &SfcCompileResult,
    lane: &Lane,
) -> bool {
    if *lane != Lane::Unrecorded {
        return false;
    }
    let code = if selected
        .errors
        .iter()
        .any(|error| error.code.as_deref() == Some("TEMPLATE_ERROR"))
    {
        "TEMPLATE_ERROR"
    } else if shape == Shape::Vapor
        && selected
            .errors
            .iter()
            .any(|error| error.code.as_deref() == Some("VAPOR_TEMPLATE_ERROR"))
    {
        "VAPOR_TEMPLATE_ERROR"
    } else {
        return false;
    };
    *tally.sfc_errors.entry(code.to_owned()).or_default() += 1;
    tally
        .prebackend_rejections
        .push((name.to_owned(), selected.errors.clone()));
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
        let refused = tally.legacy.get(EMIT_REFUSED).copied().unwrap_or(0);
        if shape.is_dom() && tally.refusals.total() != refused {
            failures.push(format!(
                "{}: {refused} emit_refused templates but {} typed refusal attributions",
                shape.id(),
                tally.refusals.total()
            ));
        }
        if (shape.is_dom() || *shape == Shape::Vapor) && tally.unrecorded != 0 {
            let backend = if shape.is_dom() { "DOM" } else { "Vapor" };
            failures.push(format!(
                "{}: {} {backend} template compiles recorded no selection counter: {:?}",
                shape.id(),
                tally.unrecorded,
                tally.unrecorded_samples
            ));
        }
    }
    failures
}

/// The Croquis projection the production selector still refuses (perf):
/// how many refused templates L2 would emit, each held to whole-module
/// parity with the legacy lane.
pub fn measure_projection(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    name: &str,
    shape: Shape,
    tally: &mut Tally,
) {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let projected = vize_atelier_dom::differential::with_croquis_projection(|| {
        compile(descriptor, name, shape)
    });
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();
    let Ok(projected) = projected else {
        return;
    };
    if classify(shape, &counters) == Ok(Lane::Accepted) {
        tally.ready += 1;
        compare_with_legacy(descriptor, name, shape, &projected, tally);
    }
}

pub fn compare_with_legacy(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    name: &str,
    shape: Shape,
    selected: &SfcCompileResult,
    tally: &mut Tally,
) {
    tally.compared += 1;
    let legacy =
        vize_atelier_dom::differential::with_legacy_lane(|| compile(descriptor, name, shape));
    if let Some(divergence) = divergence(selected, &legacy) {
        tally
            .divergences
            .push(format!("{name} [{}]: {divergence}", shape.id()));
    }
}
