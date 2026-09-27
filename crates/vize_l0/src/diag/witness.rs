//! Witness chains: the facts a proven diagnostic rests on.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use alloc::vec::Vec;

use crate::id::AnalysisId;
use crate::span::Span;

/// One fact a diagnostic depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitnessLink {
    /// The analysis that produced the fact.
    pub analysis: AnalysisId,
    /// Where the fact applies.
    pub span: Span,
}

/// A non-empty chain of [`WitnessLink`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessChain {
    links: Vec<WitnessLink>,
}

impl WitnessChain {
    /// A chain of one link.
    #[must_use]
    pub fn new(first: WitnessLink) -> Self {
        let _ = first;
        todo!()
    }

    /// Append a link.
    #[must_use]
    pub fn then(self, link: WitnessLink) -> Self {
        let _ = link;
        todo!()
    }

    /// The links in order.
    #[must_use]
    pub fn links(&self) -> &[WitnessLink] {
        &self.links
    }
}
