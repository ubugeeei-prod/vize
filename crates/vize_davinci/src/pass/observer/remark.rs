//! Optimization remarks (P3-13): a pass saying why it did or did not do
//! something, through the observer channel.
//!
//! A remark is LLVM's `-Rpass` / `-Rpass-missed` / `-Rpass-analysis` in
//! shape - `{pass, kind, name, span, args}` - with **structured** args, since
//! free-form strings are unfilterable (LLVM's own regret). The contract,
//! including the per-pass vocabulary, lives in
//! `docs/davinci/plan/remarks-format.md`.
//!
//! # The channel is the observer
//!
//! A pass body never talks to a side channel. [`run_pipeline_remarked`]
//! hands each step a [`PassRemarks`] bound to the running pass's
//! [`PassEvent`], and [`RemarkSink::emit`] forwards to
//! [`PassObserver::on_remark`] with that event - so the `pass` a remark is
//! attributed to comes from the pass manager, never from the pass's own
//! say-so, and every observer composition (`Pair`) sees remarks exactly as
//! it sees every other hook.
//!
//! # Zero cost when nothing listens
//!
//! [`PassObserver::REMARKS`] is an associated **const**, so
//! [`RemarkSink::ENABLED`] is known at monomorphization time. A pass guards
//! argument construction with `if S::ENABLED` (or [`RemarkSink::enabled`]),
//! and under an observer that does not consume remarks the whole remark path
//! (blocker classification, argument arrays, the emit call) compiles away.
//! The claim is pinned by `tests/remark_zero_cost.rs` (exact zero
//! allocations under a counting global allocator) rather than asserted.
//!
//! [`run_pipeline_remarked`]: super::run_pipeline_remarked

pub mod record;

pub use record::{RecordedArg, RecordedRemark, RemarkArgValue, RemarkCollector};

use vize_s0::Span;

use super::{PassEvent, PassObserver};

/// What a remark says about its pass's decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RemarkKind {
    /// The pass performed - or, for an analysis pass, established - the
    /// optimization the remark names.
    Applied,
    /// The pass considered the optimization and it did not apply; the args
    /// name the blocker.
    Missed,
    /// A neutral fact that explains other decisions, with no verdict.
    Analysis,
}

impl RemarkKind {
    /// Every kind, in declaration order.
    pub const ALL: [Self; 3] = [Self::Applied, Self::Missed, Self::Analysis];

    /// The stable spelling every serialized form uses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Missed => "missed",
            Self::Analysis => "analysis",
        }
    }

    /// Read a stable spelling back.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == name)
    }
}

/// One structured argument value, borrowed from the emitting pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemarkValue<'a> {
    /// Text: a closed-vocabulary token or an authored name.
    Str(&'a str),
    /// A count or an index.
    Int(i64),
    /// A predicate.
    Bool(bool),
}

/// One `key=value` argument. Keys are static so a filter can rely on them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemarkArg<'a> {
    /// The argument's name: lowercase kebab-case, unique within a remark.
    pub key: &'static str,
    /// Its value.
    pub value: RemarkValue<'a>,
}

impl<'a> RemarkArg<'a> {
    /// A text argument.
    #[must_use]
    pub const fn str(key: &'static str, value: &'a str) -> Self {
        Self {
            key,
            value: RemarkValue::Str(value),
        }
    }

    /// An integer argument.
    #[must_use]
    pub const fn int(key: &'static str, value: i64) -> Self {
        Self {
            key,
            value: RemarkValue::Int(value),
        }
    }

    /// A boolean argument.
    #[must_use]
    pub const fn bool(key: &'static str, value: bool) -> Self {
        Self {
            key,
            value: RemarkValue::Bool(value),
        }
    }
}

/// A remark as a pass emits it: borrowed and allocation-free.
///
/// The pass and stage are deliberately absent: the channel supplies them
/// from the running [`PassEvent`], so a pass cannot misattribute itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remark<'a> {
    /// Applied, missed, or analysis.
    pub kind: RemarkKind,
    /// The remark's identity within its pass (LLVM's remark name):
    /// lowercase kebab-case, from the pass's registered vocabulary.
    pub name: &'static str,
    /// The authored span the decision is about, as byte offsets into the
    /// source the pipeline's artifact was lowered from.
    pub span: Span,
    /// Structured arguments, in the pass's declared order.
    pub args: &'a [RemarkArg<'a>],
}

impl<'a> Remark<'a> {
    /// An [`Applied`](RemarkKind::Applied) remark.
    #[must_use]
    pub const fn applied(name: &'static str, span: Span, args: &'a [RemarkArg<'a>]) -> Self {
        Self {
            kind: RemarkKind::Applied,
            name,
            span,
            args,
        }
    }

    /// A [`Missed`](RemarkKind::Missed) remark.
    #[must_use]
    pub const fn missed(name: &'static str, span: Span, args: &'a [RemarkArg<'a>]) -> Self {
        Self {
            kind: RemarkKind::Missed,
            name,
            span,
            args,
        }
    }

    /// An [`Analysis`](RemarkKind::Analysis) remark.
    #[must_use]
    pub const fn analysis(name: &'static str, span: Span, args: &'a [RemarkArg<'a>]) -> Self {
        Self {
            kind: RemarkKind::Analysis,
            name,
            span,
            args,
        }
    }
}

/// Where a pass body emits remarks.
///
/// Generic rather than `dyn` for the same reason [`PassObserver`] is: the
/// detached case must compile to nothing, which a vtable call cannot.
pub trait RemarkSink {
    /// Whether an emitted remark reaches anything. Guard argument
    /// construction on it; it is a constant, so the guarded code vanishes
    /// when it is `false`.
    const ENABLED: bool;

    /// Emit one remark. A no-op when [`ENABLED`](Self::ENABLED) is false.
    fn emit(&mut self, remark: &Remark<'_>);

    /// [`ENABLED`](Self::ENABLED) as a method, for call sites holding a
    /// value rather than naming the type.
    #[inline]
    #[must_use]
    fn enabled(&self) -> bool {
        Self::ENABLED
    }
}

/// The detached sink, for pass bodies run outside the pass manager (the
/// production DOM path folds preserving passes before emission).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoRemarks;

impl RemarkSink for NoRemarks {
    const ENABLED: bool = false;

    #[inline]
    fn emit(&mut self, _remark: &Remark<'_>) {}
}

/// The pass manager's remark channel for one running pass.
///
/// Built by [`run_pipeline_remarked`](super::run_pipeline_remarked) around
/// each step; forwards to the observer's [`PassObserver::on_remark`] with
/// the pass's own event.
#[derive(Debug)]
pub struct PassRemarks<'r, 'p, O> {
    observer: &'r mut O,
    event: PassEvent<'p>,
}

impl<'r, 'p, O: PassObserver> PassRemarks<'r, 'p, O> {
    pub(super) fn new(observer: &'r mut O, event: PassEvent<'p>) -> Self {
        Self { observer, event }
    }

    /// The running pass's event.
    #[must_use]
    pub const fn event(&self) -> &PassEvent<'p> {
        &self.event
    }
}

impl<O: PassObserver> RemarkSink for PassRemarks<'_, '_, O> {
    const ENABLED: bool = O::REMARKS;

    #[inline]
    fn emit(&mut self, remark: &Remark<'_>) {
        if O::REMARKS {
            self.observer.on_remark(&self.event, remark);
        }
    }
}

/// Counts remarks by kind and stores nothing - the allocation-free
/// consumer, for gates that only need the totals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RemarkCounter {
    /// Applied remarks seen.
    pub applied: u32,
    /// Missed remarks seen.
    pub missed: u32,
    /// Analysis remarks seen.
    pub analysis: u32,
}

impl RemarkCounter {
    /// A zeroed counter.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            applied: 0,
            missed: 0,
            analysis: 0,
        }
    }

    /// Total remarks seen.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.applied + self.missed + self.analysis
    }
}

impl PassObserver for RemarkCounter {
    const REMARKS: bool = true;

    fn on_remark(&mut self, _event: &PassEvent<'_>, remark: &Remark<'_>) {
        match remark.kind {
            RemarkKind::Applied => self.applied += 1,
            RemarkKind::Missed => self.missed += 1,
            RemarkKind::Analysis => self.analysis += 1,
        }
    }
}
