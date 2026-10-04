//! One open fused walk, carrying a mark supplied by its actual caller.
//!
//! This state reads no clock and records no metric. Native callers can supply
//! an owned Timer; a browser can supply its already sampled host timestamp.
//! Only the pipeline's existing group entry/exit decides a walk boundary.

use crate::profiler::SpanAttribution;

use super::super::PassEvent;

/// A fused walk's optional caller-owned mark and lead-pass attribution.
///
/// This is observation state, not source or artifact completion authority.
/// The caller discards it on pipeline restart/failure and decides how to
/// consume a completed mark. No clock, allocation, IO or serialization occurs.
#[derive(Debug, Clone, Copy)]
pub struct WalkTiming<Mark> {
    open: Option<WalkSpan<Mark>>,
}

#[derive(Debug, Clone, Copy)]
struct WalkSpan<Mark> {
    mark: Mark,
    attribution: SpanAttribution,
}

impl<Mark> Default for WalkTiming<Mark> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Mark> WalkTiming<Mark> {
    /// Empty timing state; the mark need not implement Default or Copy.
    #[must_use]
    pub const fn new() -> Self {
        Self { open: None }
    }

    /// Discard an interrupted or failed walk without returning a sample.
    #[inline]
    pub fn discard(&mut self) {
        self.open = None;
    }

    /// At true group entry, ask the caller lazily for a mark.
    ///
    /// Non-entry passes never invoke `start`. A declined mark supplies no
    /// sample; restart/failure uses [`Self::discard`] to clear earlier state.
    #[inline]
    pub fn begin(&mut self, event: &PassEvent<'_>, start: impl FnOnce() -> Option<Mark>) {
        if !event.is_group_entry() {
            return;
        }
        if let Some(mark) = start() {
            self.open = Some(WalkSpan {
                mark,
                attribution: SpanAttribution::new()
                    .with_stage(event.pipeline.stage)
                    .with_pass(
                        (event.pipeline.passes.get(event.group.start)).map_or("", |pass| pass.name),
                    ),
            });
        }
    }

    /// At group exit, transfer the owned mark and entry attribution once.
    ///
    /// Intermediate passes retain the open mark. The caller measures or
    /// records only this returned mark; no timing operation is performed here.
    #[inline]
    #[must_use]
    pub fn end(&mut self, event: &PassEvent<'_>) -> Option<(Mark, SpanAttribution)> {
        if !event.is_group_exit() {
            return None;
        }
        self.open.take().map(|span| (span.mark, span.attribution))
    }
}
