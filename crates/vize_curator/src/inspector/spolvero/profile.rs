//! The ladder's step timings as a P0-11 profile export (C-3).
//!
//! Spolvero's timing artifact is the profiler export
//! (`davinci-road/plan/profile-export.schema.json`), not a private shape: the
//! steps a [`LadderRun`](super::LadderRun) timed are recorded into a local
//! [`Profiler`] under one dotted key with `{stage, pass, block}` attribution
//! and serialized by the profiler's own exporter - the one serializer of that
//! schema.
//!
//! The durations come from the host clock the run was given, which is why
//! this works in the browser: nothing here reads `std::time::Instant` (absent
//! on `wasm32-unknown-unknown`); the profiler only aggregates durations it is
//! handed.

use core::time::Duration;

use vize_s0::profiler::{ProfileExportBudget, ProfileExportOptions, Profiler, SpanAttribution};

use super::ladder::LadderStep;

/// The dotted key every ladder step records under.
pub const LADDER_STEP_KEY: &str = "davinci.spolvero.step";

/// `steps` as a profile export document for `command`.
///
/// Enabling a profiler resets the process's allocation-profiling window (a
/// global), so this is for hosts that are not themselves running a
/// `--profile-json` session - the wasm playground build is the caller.
///
/// # Panics
///
/// Never in practice: the exporter emits valid JSON by construction.
#[must_use]
pub fn ladder_profile(command: &'static str, steps: &[LadderStep]) -> serde_json::Value {
    let profiler = Profiler::enabled();
    for step in steps {
        let attribution = SpanAttribution::new()
            .with_stage(step.stage)
            .with_pass(step.pass)
            .with_block("template");
        profiler.record_attributed(
            LADDER_STEP_KEY,
            attribution,
            Duration::from_nanos(step.nanos),
        );
    }
    let export = profiler.export_report(&ProfileExportOptions {
        command,
        allocation: None,
        budget: ProfileExportBudget::default(),
    });
    profiler.disable();
    serde_json::from_str(export.to_json().as_str())
        .expect("the profile exporter emits valid JSON by construction")
}
