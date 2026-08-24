//! [`PassObserver`] — the seven hooks a pipeline run reports through.
//!
//! # Fused groups are reported as one walk
//!
//! The law this module exists to hold: **a fused pass's cost is never
//! attributed to a walk it did not make.** The pass manager fuses adjacent
//! fusable passes into one traversal, so three fused passes are one walk, not
//! three — and an observer that opened a span per pass would report three
//! walks' worth of overhead that never happened, then be believed.
//!
//! Every [`PassEvent`] therefore carries the [`FusionGroup`] its pass landed
//! in, and [`PassEvent::is_group_entry`] / [`PassEvent::is_group_exit`] mark
//! the pass that opens and the pass that closes the walk.
//! [`PassEvent::group_members`] names every pass sharing it. An observer that
//! wants per-walk accounting keys on `group_index` and opens at the entry;
//! one that wants per-pass accounting keys on `pass_index`. Both are
//! available, neither is guessed.
//!
//! # Zero cost when nothing is attached
//!
//! Observers are dispatched **statically**: [`run_pipeline`] is generic over
//! the observer type, so [`NoObserver`]'s empty bodies inline away and the
//! un-observed pipeline is the pipeline. That is stronger than checking
//! attachment once per pipeline (guardrail 1 asks for once per pipeline, never
//! per node) — there is no check at all. The `davinci` bench pair pins it:
//! the observer-absent case is **alloc-identical** to the un-observed one.
//!
//! No `dyn PassObserver` is provided on purpose. A trait object would put a
//! vtable call on the per-pass path and make the zero-cost claim untestable;
//! a caller that genuinely needs runtime choice composes with an enum
//! observer of its own.

pub mod budget;
pub mod folio;
pub mod remark;
pub mod timing;

pub use budget::BudgetObserver;
pub use folio::FolioObserver;
pub use remark::{Remark, RemarkSink};
pub use timing::TimingObserver;

use super::{FusionGroup, PassDesc, Pipeline};

/// One pass, in the walk it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassEvent<'a> {
    /// The pipeline being run.
    pub pipeline: &'a Pipeline,
    /// Index of the fusion group this pass runs in. One walk per group, so
    /// this is also the walk index.
    pub group_index: usize,
    /// The group itself.
    pub group: FusionGroup,
    /// Index of this pass in `pipeline.passes`.
    pub pass_index: usize,
}

impl<'a> PassEvent<'a> {
    /// This pass's description.
    #[inline]
    #[must_use]
    pub fn desc(&self) -> PassDesc {
        self.pipeline.passes[self.pass_index]
    }

    /// Whether this pass opens the walk.
    #[inline]
    #[must_use]
    pub const fn is_group_entry(&self) -> bool {
        self.pass_index == self.group.start
    }

    /// Whether this pass closes the walk.
    #[inline]
    #[must_use]
    pub const fn is_group_exit(&self) -> bool {
        self.pass_index + 1 == self.group.end()
    }

    /// Every pass sharing this walk, in order.
    ///
    /// This is what makes "one walk, member passes named" reportable rather
    /// than merely true.
    #[inline]
    #[must_use]
    pub fn group_members(&self) -> &'a [PassDesc] {
        &self.pipeline.passes[self.group.start..self.group.end()]
    }
}

/// One analysis computed during a pipeline run.
///
/// Analyses are reported separately from passes because their cost is a
/// property of the *fact* rather than of the pass that happened to need it
/// first: two passes requiring the same analysis pay for one computation, and
/// attributing it to whichever pass ran earlier would make the second look
/// free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisEvent<'a> {
    /// The pipeline being run.
    pub pipeline: &'a Pipeline,
    /// The analysis's stable name.
    pub name: &'static str,
    /// The pass that required it, if it was required by one.
    pub required_by: Option<usize>,
}

/// A pipeline run that stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailEvent<'a> {
    /// The pipeline being run.
    pub pipeline: &'a Pipeline,
    /// The pass that failed, if the failure is attributable to one.
    pub pass_index: Option<usize>,
    /// Why it stopped. Static text: a failure report must not allocate on the
    /// path that is already going wrong.
    pub reason: &'static str,
}

/// The seven hooks a pipeline run reports through.
///
/// Every method has an empty default body, so an observer implements only what
/// it consumes and gaining a hook never breaks an implementation.
pub trait PassObserver {
    /// The run is about to start.
    fn before_pipeline(&mut self, _pipeline: &Pipeline) {}
    /// The run finished without failing.
    fn after_pipeline(&mut self, _pipeline: &Pipeline) {}
    /// A pass is about to run. Check [`PassEvent::is_group_entry`] to open a
    /// walk-scoped measurement.
    fn before_pass(&mut self, _event: &PassEvent<'_>) {}
    /// A pass finished. Check [`PassEvent::is_group_exit`] to close a
    /// walk-scoped measurement.
    fn after_pass(&mut self, _event: &PassEvent<'_>) {}
    /// An analysis is about to be computed.
    fn before_analysis(&mut self, _event: &AnalysisEvent<'_>) {}
    /// An analysis finished.
    fn after_analysis(&mut self, _event: &AnalysisEvent<'_>) {}
    /// The run stopped. `after_pipeline` does **not** also fire: a failed run
    /// did not finish, and an observer that saw both could not tell the two
    /// apart.
    fn on_fail(&mut self, _event: &FailEvent<'_>) {}
}

/// The observer that observes nothing.
///
/// Every method is the empty default, so a `run_pipeline::<NoObserver, _>` call
/// monomorphizes to the pipeline with no hooks in it at all.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoObserver;

impl PassObserver for NoObserver {}

/// Two observers, both notified, in order.
///
/// Composition is a type rather than a `Vec<Box<dyn PassObserver>>` so the
/// static-dispatch property survives it: `Pair<TimingObserver, BudgetObserver>`
/// inlines to both bodies with no vtable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A: PassObserver, B: PassObserver> PassObserver for Pair<A, B> {
    fn before_pipeline(&mut self, pipeline: &Pipeline) {
        self.0.before_pipeline(pipeline);
        self.1.before_pipeline(pipeline);
    }
    fn after_pipeline(&mut self, pipeline: &Pipeline) {
        self.0.after_pipeline(pipeline);
        self.1.after_pipeline(pipeline);
    }
    fn before_pass(&mut self, event: &PassEvent<'_>) {
        self.0.before_pass(event);
        self.1.before_pass(event);
    }
    fn after_pass(&mut self, event: &PassEvent<'_>) {
        self.0.after_pass(event);
        self.1.after_pass(event);
    }
    fn before_analysis(&mut self, event: &AnalysisEvent<'_>) {
        self.0.before_analysis(event);
        self.1.before_analysis(event);
    }
    fn after_analysis(&mut self, event: &AnalysisEvent<'_>) {
        self.0.after_analysis(event);
        self.1.after_analysis(event);
    }
    fn on_fail(&mut self, event: &FailEvent<'_>) {
        self.0.on_fail(event);
        self.1.on_fail(event);
    }
}

/// Why a pipeline run stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassFailure {
    /// Static text: the failure path must not allocate.
    pub reason: &'static str,
}

impl PassFailure {
    /// A failure with `reason`.
    #[inline]
    #[must_use]
    pub const fn new(reason: &'static str) -> Self {
        Self { reason }
    }
}

/// Run `pipeline`'s plan, firing `observer`'s hooks around each pass.
///
/// `step` is the pass body. P2-2's non-goal stands — there is no S2 to run
/// over until P2-5a, and no catalogue binding a name to an implementation
/// until P2-9 — so the *plan* is executed here and the bodies are the
/// caller's. That is also what makes the zero-cost claim measurable before any
/// real pass exists.
///
/// # Errors
///
/// Returns the first [`PassFailure`] a step reports, after firing `on_fail`.
/// `after_pipeline` does not fire on that path.
pub fn run_pipeline<O, F>(
    pipeline: &Pipeline,
    observer: &mut O,
    mut step: F,
) -> Result<(), PassFailure>
where
    O: PassObserver,
    F: FnMut(&PassEvent<'_>) -> Result<(), PassFailure>,
{
    observer.before_pipeline(pipeline);

    let group_count = pipeline.group_count();
    let mut group_index = 0;
    while group_index < group_count {
        let group = pipeline
            .group(group_index)
            .expect("group index below group_count always resolves");
        let mut pass_index = group.start;
        while pass_index < group.end() {
            let event = PassEvent {
                pipeline,
                group_index,
                group,
                pass_index,
            };
            observer.before_pass(&event);
            match step(&event) {
                Ok(()) => observer.after_pass(&event),
                Err(failure) => {
                    observer.on_fail(&FailEvent {
                        pipeline,
                        pass_index: Some(pass_index),
                        reason: failure.reason,
                    });
                    return Err(failure);
                }
            }
            pass_index += 1;
        }
        group_index += 1;
    }

    observer.after_pipeline(pipeline);
    Ok(())
}
