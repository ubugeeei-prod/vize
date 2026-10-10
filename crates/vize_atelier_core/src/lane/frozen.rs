//! Compiler-owned join of parser frozen-element provenance into the existing lane.

use super::{TransformContext, TransformLaneOptions, transform_inner};
use crate::{CompilerError, ErrorCode, RootNode, TransformOptions, options::CustomElementMatcher};
use vize_armature::FrozenElements;
use vize_croquis::Croquis;
use vize_l0::{Allocator, Span, String};

/// Join an original parser bundle before any transform/context mutation.
///
/// The trusted compiler passes its own parser root and original borrowed source.
/// This checks both source identities, not the integrity of a publicly mutable AST.
#[doc(hidden)]
#[expect(
    clippy::too_many_arguments,
    reason = "existing compiler options plus source-bound custody"
)]
pub fn transform_with_frozen_elements<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &mut RootNode<'a>,
    options: TransformOptions,
    analysis: Option<&'a Croquis>,
    custom_elements: CustomElementMatcher,
    template_syntax_quirks: bool,
    hoisted_scope_id: Option<String>,
    frozen: FrozenElements<'a>,
) -> std::vec::Vec<CompilerError> {
    let Some(spans) = frozen.spans_for(root.source, source) else {
        return std::vec![CompilerError::with_message(
            ErrorCode::ExtendPoint,
            "Frozen element provenance belongs to a different template source.",
            None,
        )];
    };
    transform_inner(
        allocator,
        root,
        options,
        analysis,
        TransformLaneOptions {
            template_syntax_quirks,
            hoisted_scope_id,
            custom_elements,
            frozen_elements: spans,
            ..Default::default()
        },
        Some(source),
    )
}

impl TransformContext<'_> {
    pub(super) fn element_is_frozen(&self, span: Span) -> bool {
        self.frozen_elements
            .binary_search_by_key(&span.start, |candidate| candidate.start)
            .is_ok_and(|index| self.frozen_elements[index] == span)
    }
}

#[cfg(test)]
mod tests;
