//! Stable semantic links emitted by virtual TypeScript generation.

use std::ops::Range;

/// A stable semantic edge between two generated TypeScript ranges that model
/// one authored Vue binding but intentionally have different TypeScript
/// symbols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VizeSemanticLink {
    pub source_range: Range<usize>,
    pub target_range: Range<usize>,
    pub kind: VizeSemanticLinkKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VizeSemanticLinkKind {
    VueSetupTemplateRefUnwrap,
    /// A generated component binding and one mapped prop access that TypeScript
    /// can use as a project-wide navigation entry point.
    VueComponentPropNavigation,
}

impl VizeSemanticLink {
    /// Shift both generated TypeScript endpoint ranges after prefixing code.
    ///
    /// The link's `source_range` names the first semantic endpoint, not an
    /// authored SFC source range. Both endpoints must stay aligned with the
    /// generated text consumed by Corsa and the content-mapper protocol.
    #[cfg(feature = "native")]
    pub(crate) fn shift_generated_ranges(&mut self, offset: usize) {
        self.source_range.start += offset;
        self.source_range.end += offset;
        self.target_range.start += offset;
        self.target_range.end += offset;
    }
}
