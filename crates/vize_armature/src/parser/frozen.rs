//! Compiler-only custody of authored elements frozen by own or inherited v-pre.

use super::Parser;
use vize_l0::{Span, Vec};
use vize_relief::{RootNode, errors::CompilerError};

/// Provenance from one existing parser walk, bound to its exact source buffer.
///
/// This trusts the original parser bundle used by a compiler owner; it does not
/// make publicly mutable AST nodes or their source field impossible to forge.
#[doc(hidden)]
#[derive(Debug)]
pub struct FrozenElements<'a> {
    source: &'a str,
    spans: &'a [Span],
}

impl<'a> FrozenElements<'a> {
    pub(super) fn new(source: &'a str, spans: &'a [Span]) -> Self {
        Self { source, spans }
    }

    /// Join only the original root and caller's exact borrowed source buffers.
    #[doc(hidden)]
    pub fn spans_for(&self, root_source: &str, caller_source: &str) -> Option<&'a [Span]> {
        (core::ptr::eq(self.source, root_source) && core::ptr::eq(self.source, caller_source))
            .then_some(self.spans)
    }
}

impl<'a> Parser<'a> {
    /// Opt in to frozen-element custody without changing the public parse tuple.
    #[doc(hidden)]
    pub fn parse_with_frozen_elements(
        mut self,
    ) -> (
        RootNode<'a>,
        std::vec::Vec<CompilerError>,
        FrozenElements<'a>,
    ) {
        self.frozen_elements = Some(Vec::new_in(&self.allocator));
        self.parse_inner()
    }

    pub(in crate::parser) fn record_frozen_element(&mut self, span: Span, frozen: bool) {
        if frozen && let Some(spans) = &mut self.frozen_elements {
            spans.push(span);
        }
    }

    pub(super) fn into_frozen_result(
        mut self,
    ) -> (
        RootNode<'a>,
        std::vec::Vec<CompilerError>,
        FrozenElements<'a>,
    ) {
        let source = self.source;
        let spans = self
            .frozen_elements
            .take()
            .map(Vec::into_arena_slice)
            .unwrap_or(&[]);
        let (root, errors) = self.into_result();
        (root, errors, FrozenElements::new(source, spans))
    }
}

#[cfg(test)]
mod tests;
