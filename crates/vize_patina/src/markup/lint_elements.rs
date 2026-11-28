//! Original element observations for document-shaped configured rules.
//!
//! Vue keeps its declared compatibility route. JSX projections read exact
//! factory facts. Native depth checks retain the original tree until original
//! lint ancestry has its own qualified native capability.

use super::element::MarkupElementInner;
use super::l2::L2ElementOp;
use super::{MarkupDocument, MarkupDocumentInner, MarkupElement, MarkupElementKind};
use crate::ir::ByteRange;
use std::cell::Cell;
use vize_relief::ElementType;

pub(crate) struct LintElement<'a> {
    element: MarkupElement<'a>,
    pub(crate) depth: u32,
}

impl LintElement<'_> {
    pub(crate) fn tag(&self) -> &str {
        self.element.tag()
    }

    pub(crate) fn is_plain(&self) -> bool {
        match self.element.inner {
            MarkupElementInner::Relief(node) => node.tag_type == ElementType::Element,
            MarkupElementInner::L2 {
                op: L2ElementOp::Component(component),
                doc,
                ..
            } => doc
                .jsx_syntax
                .is_some_and(|syntax| syntax.component_was_intrinsic(component)),
            _ => self.element.kind() == MarkupElementKind::Element,
        }
    }

    pub(crate) fn range(&self) -> ByteRange {
        match self.element.inner {
            MarkupElementInner::Relief(node) => super::loc_to_range(&node.loc),
            MarkupElementInner::L2 {
                op: L2ElementOp::Element(element),
                doc,
                ..
            } => doc
                .jsx_syntax
                .and_then(|syntax| syntax.slot_diagnostic_range(element))
                .map_or_else(|| self.element.range(), super::l2_range),
            _ => self.element.range(),
        }
    }
}

impl<'a> MarkupDocument<'a> {
    pub(crate) fn walk_lint_elements(
        &self,
        require_original_ancestry: bool,
        visitor: &mut impl FnMut(LintElement<'a>),
    ) {
        if require_original_ancestry
            && let MarkupDocumentInner::L2(doc) = self.inner
            && let Some(syntax) = doc.jsx_syntax
            && syntax.lowered().l2.as_ref().is_ok_and(|root| root.native)
        {
            // Exact original lint ancestry has not been qualified as a native
            // capability. Keep this explicit retained-tree fallback rather
            // than infer that authority from semantic op spans.
            MarkupDocument::new(&syntax.lowered().root, self.syntax)
                .walk_lint_elements(false, visitor);
            return;
        }
        let depth = Cell::new(1u32);
        self.walk_tree(
            &mut |element| {
                visitor(LintElement {
                    element,
                    depth: depth.get(),
                });
                depth.set(depth.get().saturating_add(1));
            },
            &mut |_| depth.set(depth.get().saturating_sub(1)),
        );
    }
}
