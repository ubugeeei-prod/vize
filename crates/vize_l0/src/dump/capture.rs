//! Optional observation of the stages that produced a compiler result.
//!
//! Level code supplies page builders as closures. [`NoCapture`] never calls
//! them, so ordinary compilation does not print or allocate dump pages.
//! A host commits the provisional pages only after its native emitter returns
//! the module; a legacy selection or a rejected emission discards them.

use alloc::vec::Vec;

use crate::{Span, String, level::Level};

/// Whether the recorded stages actually produced the returned module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureOutcome {
    /// The product has not selected or emitted a backend yet.
    Pending,
    /// The recorded native stages produced the returned module.
    Accepted,
    /// A compatibility backend produced the returned module instead.
    Legacy(String),
    /// The returned module has no template stages to capture.
    Unavailable(String),
    /// Compilation refused an artifact; no module was returned.
    Rejected(String),
}

/// An executed level boundary or pass, rendered only for an observed run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageCapturePage {
    pub level: Level,
    pub step: &'static str,
    pub text: String,
}

/// An option that affected the product result, recorded at the host boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureOption {
    pub name: &'static str,
    pub value: String,
}

/// Elapsed host-clock time for one stage that actually ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageTiming {
    pub level: Level,
    pub step: &'static str,
    pub nanos: u64,
}

/// A structured argument in a pass remark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureArgValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureArg {
    pub name: String,
    pub value: CaptureArgValue,
}

/// A remark from a pass that the product compile really executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureRemark {
    pub level: Level,
    pub pass: String,
    pub kind: String,
    pub name: String,
    pub span: Span,
    pub args: Vec<CaptureArg>,
}

/// The sidecar for one product compile. It is never transported between levels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageCapture {
    /// An opaque host-selected output target (for example a backend name).
    pub target: String,
    pub outcome: CaptureOutcome,
    pub options: Vec<CaptureOption>,
    pub pages: Vec<StageCapturePage>,
    pub timings: Vec<StageTiming>,
    pub remarks: Vec<CaptureRemark>,
}

impl StageCapture {
    #[must_use]
    pub fn new(target: &str) -> Self {
        Self {
            target: String::from(target),
            outcome: CaptureOutcome::Pending,
            options: Vec::new(),
            pages: Vec::new(),
            timings: Vec::new(),
            remarks: Vec::new(),
        }
    }

    /// Record an effective compile option beside the outcome.
    pub fn option(&mut self, name: &'static str, value: impl Into<String>) {
        self.options.push(CaptureOption {
            name,
            value: value.into(),
        });
    }
}

/// A compile-time selected page sink. Pass a renderer rather than preprinted
/// text so the ordinary [`NoCapture`] specialization does no dump work.
pub trait CaptureSink {
    /// Compile-time switch for the ordinary emitter's original hot path.
    const RECORDING: bool;

    fn page<F: FnOnce() -> String>(&mut self, level: Level, step: &'static str, render: F);

    /// Record a measured host-clock window. No clock is read by this trait.
    fn timing<F: FnOnce() -> u64>(&mut self, level: Level, step: &'static str, nanos: F);

    fn remark<F: FnOnce() -> CaptureRemark>(&mut self, render: F);

    /// Commit pages only when the native backend emitted the returned module.
    fn finish<F: FnOnce() -> CaptureOutcome>(&mut self, outcome: F);
}

/// A zero-size sink for ordinary compiles.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoCapture;

impl CaptureSink for NoCapture {
    const RECORDING: bool = false;

    #[inline(always)]
    fn page<F: FnOnce() -> String>(&mut self, _: Level, _: &'static str, _: F) {}

    #[inline(always)]
    fn timing<F: FnOnce() -> u64>(&mut self, _: Level, _: &'static str, _: F) {}

    #[inline(always)]
    fn remark<F: FnOnce() -> CaptureRemark>(&mut self, _: F) {}

    #[inline(always)]
    fn finish<F: FnOnce() -> CaptureOutcome>(&mut self, _: F) {}
}

impl CaptureSink for StageCapture {
    const RECORDING: bool = true;

    fn page<F: FnOnce() -> String>(&mut self, level: Level, step: &'static str, render: F) {
        self.pages.push(StageCapturePage {
            level,
            step,
            text: render(),
        });
    }

    fn timing<F: FnOnce() -> u64>(&mut self, level: Level, step: &'static str, nanos: F) {
        self.timings.push(StageTiming {
            level,
            step,
            nanos: nanos(),
        });
    }

    fn remark<F: FnOnce() -> CaptureRemark>(&mut self, render: F) {
        self.remarks.push(render());
    }

    fn finish<F: FnOnce() -> CaptureOutcome>(&mut self, outcome: F) {
        let outcome = outcome();
        if !matches!(outcome, CaptureOutcome::Accepted) {
            self.pages.clear();
            self.timings.clear();
            self.remarks.clear();
        }
        self.outcome = outcome;
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::{CaptureOutcome, CaptureSink, NoCapture, StageCapture};
    use crate::{String, level::Level};

    #[test]
    fn unobserved_run_never_renders_page() {
        let rendered = Cell::new(false);
        NoCapture.page(Level::L1, "parse", || {
            rendered.set(true);
            String::from("page")
        });
        assert!(!rendered.get());
    }

    #[test]
    fn fallback_discards_provisional_native_pages() {
        let mut capture = StageCapture::new("dom");
        capture.page(Level::L1, "parse", || String::from("page"));
        capture.finish(|| CaptureOutcome::Legacy(String::from("unsupported")));
        assert!(capture.pages.is_empty());
        assert_eq!(
            capture.outcome,
            CaptureOutcome::Legacy(String::from("unsupported"))
        );
    }
}

// Diagnostic-only source edit for the guarded PR tier.
