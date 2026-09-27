//! Per-pass dump collection driven by the pass manager.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::pass::observer::PassEvent;

/// The canonical text of an artifact after one pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DumpPage {
    /// The pass that produced this state; `None` for the seed.
    pub pass: Option<&'static str>,
    /// Canonical `Full` text.
    pub text: Box<str>,
}

/// Collects [`DumpPage`]s as a pipeline runs. Unobserved runs never build one.
#[derive(Debug, Clone, Default)]
pub struct DumpRuntime {
    after_change_only: bool,
    pages: Vec<DumpPage>,
}

impl DumpRuntime {
    /// A runtime that records every pass, or only passes that changed the text.
    #[must_use]
    pub const fn new(after_change_only: bool) -> Self {
        Self {
            after_change_only,
            pages: Vec::new(),
        }
    }

    /// Record the artifact before the first pass.
    pub fn seed(&mut self, canonical_text: &str) {
        let _ = canonical_text;
        todo!()
    }

    /// Record the artifact after the pass in `event`.
    pub fn after_pass(&mut self, event: &PassEvent<'_>, canonical_text: &str) {
        let _ = (event, canonical_text, self.after_change_only);
        todo!()
    }

    /// The pages recorded so far.
    #[must_use]
    pub fn pages(&self) -> &[DumpPage] {
        &self.pages
    }
}
