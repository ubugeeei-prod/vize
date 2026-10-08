//! The timing observer: one profile span per **walk**, not per pass.
//!
//! Costs are recorded through `vize_l0::profiler`, whose export is P0-11's
//! [`profile-export.schema.json`](../../../../docs/davinci/plan/profile-export.schema.json).
//! Attribution reuses `vize_l0::profiler::SpanAttribution` — the
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
//! [`PassEvent::is_group_entry`](vize_l0::pass::PassEvent::is_group_entry) and closes
//! at [`is_group_exit`](vize_l0::pass::PassEvent::is_group_exit), and its attribution
//! names the pass that **leads** the group. Which passes shared the walk is
//! recoverable from the pipeline itself
//! ([`PassEvent::group_members`](vize_l0::pass::PassEvent::group_members)); what is not
//! recoverable, once thrown away, is the fact that they shared it.
//!
//! # Cost when profiling is off
//!
//! `vize_l0::profiler` is globally gated by one relaxed atomic load, and
//! this observer takes no timestamp until it has passed that check. Attaching
//! the observer with profiling disabled therefore costs one atomic load per
//! walk — not per node, and not per pass.

use vize_l0::pass::{FailEvent, PassEvent, PassObserver, Pipeline, WalkTiming};
use vize_l0::profiler::global_profiler;

/// Records one profile span per fused walk.
#[derive(Debug)]
pub struct TimingObserver {
    /// The dotted span key every walk records under.
    key: &'static str,
    /// The open walk's guard, if profiling is on and a walk is open.
    open: WalkTiming<vize_l0::profiler::Timer>,
    /// Walks whose span was actually recorded (profiling on).
    pub recorded_walks: u32,
}

impl Default for TimingObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl TimingObserver {
    /// The default davinci key walks record under. A host that times walks
    /// with its own clock (Spolvero in the browser) records under the same
    /// key, so a profile reader has one spelling for "a walk".
    pub const WALK_KEY: &'static str = "davinci.pass.walk";

    /// A timing observer recording under the default davinci key.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_key(Self::WALK_KEY)
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
            open: WalkTiming::new(),
            recorded_walks: 0,
        }
    }
}

impl PassObserver for TimingObserver {
    fn before_pipeline(&mut self, _pipeline: &Pipeline) {
        // A pipeline starting while a walk is open means the previous run was
        // abandoned mid-walk. Drop the stale timer rather than record it: a
        // duration that spans two runs is worse than a missing one.
        self.open.discard();
    }

    fn before_pass(&mut self, event: &PassEvent<'_>) {
        let key = self.key;
        self.open.begin(event, || {
            let profiler = global_profiler();
            // One relaxed atomic load; no timestamp is taken when profiling is
            // off, which is what keeps an attached-but-disabled observer cheap.
            profiler.timer(key)
        });
    }

    fn after_pass(&mut self, event: &PassEvent<'_>) {
        if let Some((timer, attribution)) = self.open.end(event) {
            let elapsed = timer.stop();
            global_profiler().record_attributed(self.key, attribution, elapsed);
            self.recorded_walks += 1;
        }
    }

    fn on_fail(&mut self, _event: &FailEvent<'_>) {
        // A failed walk's duration measures how long it took to fail, which is
        // not the cost of the walk. Discard it rather than record a number
        // whose meaning differs from every other sample under the same key.
        self.open.discard();
    }
}
