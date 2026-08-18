//! The remark sink: where optimization remarks will go.
//!
//! A remark is a pass saying why it did or did not do something — LLVM's
//! `-Rpass-missed` in shape. The *content* is P3-13's; what lands here is the
//! channel, so a pass written before P3-13 has somewhere to emit and the
//! decision about what to say is not also a decision about how to say it.
//!
//! [`CountingRemarkSink`] is the no-op default: it accepts remarks and counts
//! them without storing text, which keeps the sink allocation-free while still
//! letting a test prove a pass emitted.

use super::{PassObserver, Pipeline};

/// One thing a pass has to say about its own decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remark {
    /// The pass that emitted it.
    pub pass: &'static str,
    /// What it did or did not do. Static text until P3-13 defines the
    /// vocabulary — a remark that allocates is a remark that will not be
    /// emitted on the hot path.
    pub message: &'static str,
    /// Whether the pass applied the transformation it is remarking about.
    pub applied: bool,
}

/// Somewhere for a pass to emit remarks.
pub trait RemarkSink {
    /// Accept one remark.
    fn remark(&mut self, remark: Remark);
}

/// The no-op sink: counts, stores nothing.
///
/// This is the default until P3-13, and it stays the default for release
/// builds afterwards — verification and explanation never ship on the hot path
/// (guardrail 5).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CountingRemarkSink {
    /// Remarks whose pass applied its transformation.
    pub applied: u32,
    /// Remarks whose pass did not.
    pub missed: u32,
}

impl CountingRemarkSink {
    /// A zeroed sink.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            applied: 0,
            missed: 0,
        }
    }

    /// Total remarks accepted.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.applied + self.missed
    }
}

impl RemarkSink for CountingRemarkSink {
    fn remark(&mut self, remark: Remark) {
        if remark.applied {
            self.applied += 1;
        } else {
            self.missed += 1;
        }
    }
}

/// A sink is also an observer, so attaching one costs the same nothing every
/// other observer costs when it is [`NoObserver`](super::NoObserver).
impl PassObserver for CountingRemarkSink {
    fn before_pipeline(&mut self, _pipeline: &Pipeline) {}
}
