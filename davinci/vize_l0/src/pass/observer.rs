//! Pass observers. Every hook defaults to a no-op so unobserved runs cost nothing.

use super::{PassDesc, PassFailure, Pipeline};

pub mod fusion;
pub mod remarks;
pub mod timing;

/// One pass boundary.
#[derive(Debug, Clone, Copy)]
pub struct PassEvent<'a> {
    /// The running pipeline.
    pub pipeline: &'a Pipeline,
    /// The pass.
    pub pass: &'a PassDesc,
    /// Its index in the pipeline.
    pub index: usize,
}

/// A remark a pass emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remark<'a> {
    /// Stable remark id.
    pub id: &'a str,
    /// Human message.
    pub message: &'a str,
}

/// Hooks the pass manager calls.
pub trait PassObserver {
    /// Before the first pass.
    fn before_pipeline(&mut self, _pipeline: &Pipeline) {}
    /// After the last pass.
    fn after_pipeline(&mut self, _pipeline: &Pipeline) {}
    /// Before one pass.
    fn before_pass(&mut self, _event: &PassEvent<'_>) {}
    /// After one pass.
    fn after_pass(&mut self, _event: &PassEvent<'_>) {}
    /// When a pass fails.
    fn on_fail(&mut self, _event: &PassEvent<'_>, _failure: &PassFailure) {}
    /// When a pass emits a remark.
    fn on_remark(&mut self, _event: &PassEvent<'_>, _remark: &Remark<'_>) {}
}

/// The observer that observes nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoObserver;

impl PassObserver for NoObserver {}
