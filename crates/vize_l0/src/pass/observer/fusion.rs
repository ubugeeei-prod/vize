//! The fusion observer: records which passes shared one walk.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use alloc::vec::Vec;

use super::{PassEvent, PassObserver};
use crate::pass::Pipeline;

/// A run of passes fused into one walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FusionGroup {
    /// Index of the first pass.
    pub start: usize,
    /// Number of passes.
    pub len: usize,
}

/// The fusion groups of `pipeline`, computed from its const data.
#[must_use]
pub fn fusion_groups(pipeline: &Pipeline) -> Vec<FusionGroup> {
    let _ = pipeline;
    todo!()
}

/// Records the fusion group each executed pass landed in.
#[derive(Debug, Clone, Default)]
pub struct FusionObserver {
    groups: Vec<FusionGroup>,
}

impl FusionObserver {
    /// The groups observed so far.
    #[must_use]
    pub fn groups(&self) -> &[FusionGroup] {
        &self.groups
    }
}

impl PassObserver for FusionObserver {
    fn before_pass(&mut self, event: &PassEvent<'_>) {
        let _ = event;
        todo!()
    }
}
