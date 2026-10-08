//! Binding metadata accessors for the projection mapping.

use super::{ProjectionMapping, VizeMapping, VizeSemanticLink};

impl ProjectionMapping {
    /// Semantic links between generated ranges.
    #[inline]
    pub fn semantic_links(&self) -> &[VizeSemanticLink] {
        &self.semantic_links
    }

    /// The rows and semantic links, dropping metadata and base.
    pub fn into_parts(self) -> (Vec<VizeMapping>, Vec<VizeSemanticLink>) {
        (self.spans, self.semantic_links)
    }

}
