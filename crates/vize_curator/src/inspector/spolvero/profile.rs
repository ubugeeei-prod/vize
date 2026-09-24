//! The ladder's timings as a P0-11 profile export (C-3).
//!
//! Spolvero's timing artifact is the profiler export
//! (`docs/davinci/plan/profile-export.schema.json`), not a private shape: the
//! steps and walks a [`LadderRun`](super::LadderRun) timed are recorded into
//! a local [`Profiler`] with `{stage, pass, block}` attribution and
//! serialized by the profiler's own exporter - the one serializer of that
//! schema. Steps record under [`LADDER_STEP_KEY`]; walks under
//! [`LADDER_WALK_KEY`], the timing observer's own key, attributed like the
//! timing observer attributes them (to the walk's lead pass).
//!
//! The durations come from the host clock the run was given, which is why
//! this works in the browser: nothing here reads `std::time::Instant` (absent
//! on `wasm32-unknown-unknown`); the profiler only aggregates durations it is
//! handed.

use core::time::Duration;

use vize_davinci::pass::TimingObserver;
use vize_s0::profiler::{ProfileExportBudget, ProfileExportOptions, Profiler, SpanAttribution};

use super::ladder::LadderStep;

/// The dotted key every ladder step records under.
pub const LADDER_STEP_KEY: &str = "davinci.spolvero.step";

/// The dotted key every walk records under: the timing observer's.
pub const LADDER_WALK_KEY: &str = TimingObserver::WALK_KEY;

/// `steps` and `walks` as a profile export document for `command`.
///
/// Enabling a profiler resets the process's allocation-profiling window (a
/// global), so this is for hosts that are not themselves running a
/// `--profile-json` session - the wasm playground build is the caller.
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
    let profiler = Profiler::enabled();
    for (key, timed) in [(LADDER_STEP_KEY, steps), (LADDER_WALK_KEY, walks)] {
        for step in timed {
            let attribution = SpanAttribution::new()
                .with_stage(step.stage)
                .with_pass(step.pass)
                .with_block("template");
            profiler.record_attributed(key, attribution, Duration::from_nanos(step.nanos));
        }
    }
    let export = profiler.export_report(&ProfileExportOptions {
        command,
        allocation: None,
        budget: ProfileExportBudget::default(),
    });
    profiler.disable();
    // The exporter emits valid JSON by construction; `null` is the
    // unreachable fallback rather than an abort.
    serde_json::from_str(export.to_json().as_str()).unwrap_or_default()
}
