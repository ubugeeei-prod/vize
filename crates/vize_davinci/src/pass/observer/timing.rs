//! The timing observer: one profile span per **walk**, not per pass.
//!
//! Costs are recorded through `vize_s0::profiler`, whose export is P0-11's
//! [`profile-export.schema.json`](../../../../davinci-road/plan/profile-export.schema.json).
//! Attribution reuses `vize_s0::profiler::SpanAttribution` — the
//! `{stage, pass, file_id, block, span}` builders that already exist for
//! exactly this — rather than introducing a second attribution model, which is
//! the failure the P0-11 scaffolding was landed early to prevent.
//!
//! # Why the span is the walk
//!
//! Fusing three passes into one traversal means the traversal happened once.
//! A timer per pass would report three walks' worth of enter/exit overhead
//! that was never paid, and — worse — would be believed, because the numbers
//! would look plausible. So the span opens at
//! [`PassEvent::is_group_entry`](super::PassEvent::is_group_entry) and closes
//! at [`is_group_exit`](super::PassEvent::is_group_exit), and its attribution
//! names the pass that **leads** the group. Which passes shared the walk is
//! recoverable from the pipeline itself
//! ([`PassEvent::group_members`](super::PassEvent::group_members)); what is not
//! recoverable, once thrown away, is the fact that they shared it.
//!
//! # Cost when profiling is off
//!
//! `vize_s0::profiler` is globally gated by one relaxed atomic load, and
//! this observer takes no timestamp until it has passed that check. Attaching
//! the observer with profiling disabled therefore costs one atomic load per
//! walk — not per node, and not per pass.

use vize_s0::profiler::{SpanAttribution, global_profiler};

use super::{FailEvent, PassEvent, PassObserver, Pipeline};

/// Records one profile span per fused walk.
#[derive(Debug, Default)]
pub struct TimingObserver {
    /// The dotted span key every walk records under.
    key: &'static str,
    /// The open walk's guard, if profiling is on and a walk is open.
    open: Option<WalkSpan>,
    /// Walks whose span was actually recorded (profiling on).
    pub recorded_walks: u32,
}

/// A walk's open timer plus the attribution it will record under.
#[derive(Debug)]
struct WalkSpan {
    timer: vize_s0::profiler::Timer,
    attribution: SpanAttribution,
}

impl TimingObserver {
    /// A timing observer recording under the default davinci key.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_key("davinci.pass.walk")
    }

    /// A timing observer recording under `key`.
    ///
    /// The key is the historical dotted identity every profile span already
    /// has; attribution extends it rather than replacing it, so an existing
    /// consumer that groups by key keeps working.
    #[must_use]
    pub const fn with_key(key: &'static str) -> Self {
        Self {
            key,
            open: None,
            recorded_walks: 0,
        }
    }

    /// The attribution a walk of `event` records under.
    ///
    /// `stage` is the pipeline's stage; `pass` is the group's **lead** pass,
    /// which is the walk's identity — the passes fused behind it are named by
    /// the pipeline, not by the span key.
    fn attribution(event: &PassEvent<'_>) -> SpanAttribution {
        SpanAttribution::new()
            .with_stage(event.pipeline.stage)
            .with_pass(event.pipeline.passes[event.group.start].name)
    }
}

impl PassObserver for TimingObserver {
    fn before_pipeline(&mut self, _pipeline: &Pipeline) {
        // A pipeline starting while a walk is open means the previous run was
        // abandoned mid-walk. Drop the stale timer rather than record it: a
        // duration that spans two runs is worse than a missing one.
        self.open = None;
    }

    fn before_pass(&mut self, event: &PassEvent<'_>) {
        if !event.is_group_entry() {
            return;
        }
        let profiler = global_profiler();
        // One relaxed atomic load; no timestamp is taken when profiling is
        // off, which is what keeps an attached-but-disabled observer cheap.
        if let Some(timer) = profiler.timer(self.key) {
            self.open = Some(WalkSpan {
                timer,
                attribution: Self::attribution(event),
            });
        }
    }

    fn after_pass(&mut self, event: &PassEvent<'_>) {
        if !event.is_group_exit() {
            return;
        }
        if let Some(span) = self.open.take() {
            let elapsed = span.timer.stop();
            global_profiler().record_attributed(self.key, span.attribution, elapsed);
            self.recorded_walks += 1;
        }
    }

    fn on_fail(&mut self, _event: &FailEvent<'_>) {
        // A failed walk's duration measures how long it took to fail, which is
        // not the cost of the walk. Discard it rather than record a number
        // whose meaning differs from every other sample under the same key.
        self.open = None;
    }
}
