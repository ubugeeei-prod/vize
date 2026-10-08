//! The ladder's timings as a P0-11 profile export (C-3).
//!
//! Spolvero's timing artifact is the profiler export
//! (`docs/davinci/plan/profile-export.schema.json`), not a private shape: the
//! steps and walks a [`LadderRun`](super::LadderRun) timed are recorded into
//! the sole local duration aggregate with `{stage, pass, block}` attribution
//! and serialized by the host exporter - the one serializer of that
//! schema. Steps record under [`LADDER_STEP_KEY`]; walks under
//! [`LADDER_WALK_KEY`], the timing observer's own key, attributed like the
//! timing observer attributes them (to the walk's lead pass).
//!
//! The durations come from the host clock the run was given, which is why
//! this works in the browser: nothing here reads `std::time::Instant` (absent
//! on `wasm32-unknown-unknown`); the existing metrics only aggregate durations
//! they are handed. This report does not enable or disable a native profiler,
//! reset its allocation window, or acquire its sharded metric locks.

use core::time::Duration;

use vize_carton::profile_export::{
    ProfileExportBudget, ProfileExportOptions, export_report_from_snapshots,
};
use vize_carton::timing_observer::TimingObserver;
use vize_l0::FxHashMap;
use vize_l0::profiler::{Metrics, SpanAttribution};

use super::ladder::LadderStep;

/// The dotted key every ladder step records under.
pub const LADDER_STEP_KEY: &str = "davinci.spolvero.step";

/// The dotted key every walk records under: the timing observer's.
pub const LADDER_WALK_KEY: &str = TimingObserver::WALK_KEY;

/// `steps` and `walks` as a profile export document for `command`.
///
/// Aggregates only the caller's measured durations. Allocation observations
/// remain unavailable (`null`), and counters remain the actual empty input.
/// The local map and report serialization can allocate; neither changes a
/// native profile session's enable flag, allocation window or stored metrics.
///
/// # Panics
///
/// Never in practice: the exporter emits valid JSON by construction.
#[must_use]
pub fn ladder_profile(
    command: &'static str,
    steps: &[LadderStep],
    walks: &[LadderStep],
) -> serde_json::Value {
    let mut spans = FxHashMap::default();
    for (key, timed) in [(LADDER_STEP_KEY, steps), (LADDER_WALK_KEY, walks)] {
        for step in timed {
            let attribution = SpanAttribution::new()
                .with_stage(step.stage)
                .with_pass(step.pass)
                .with_block("template");
            spans
                .entry((key, attribution))
                .or_insert_with(Metrics::new)
                .record(Duration::from_nanos(step.nanos));
        }
    }
    let export = export_report_from_snapshots(
        spans
            .into_iter()
            .map(|((key, attribution), metrics)| (key, attribution, metrics))
            .collect(),
        Vec::new(),
        &ProfileExportOptions {
            command,
            allocation: None,
            budget: ProfileExportBudget::default(),
        },
    );
    // The exporter emits valid JSON by construction; `null` is the
    // unreachable fallback rather than an abort.
    serde_json::from_str(export.to_json().as_str()).unwrap_or_default()
}
