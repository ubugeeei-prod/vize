//! A borrowed view of a node's children as Rendu ops.

use crate::{TemplateChildNode, rendu::RenduOp};

/// A borrowed, allocation-free view of a node's children as Rendu ops.
///
/// Wraps the Relief child slice without copying; iterating yields each child's
/// [`RenduOp`] paired with the borrowed node, so the emitter can drive child
/// emission from the Rendu stream while still reaching the Relief node for the
/// details the op does not yet carry (#1756).
#[derive(Debug, Clone, Copy)]
pub struct RenduChildren<'a> {
    children: &'a [TemplateChildNode<'a>],
}

impl<'a> RenduChildren<'a> {
    /// View a borrowed child slice as Rendu ops.
    pub const fn new(children: &'a [TemplateChildNode<'a>]) -> Self {
        Self { children }
    }

    /// Iterate `(op, node)` pairs for the children visible to codegen.
    ///
    /// Directive comments (`@vize:`), which never emit, are skipped — exactly
    /// as the emitter's `is_directive_comment` filter does — so a caller groups
    /// and separates the same child set it does today.
    pub fn rendered(self) -> impl Iterator<Item = (RenduOp<'a>, &'a TemplateChildNode<'a>)> {
        self.children.iter().filter_map(|child| {
            let op = RenduOp::from_template_child(child);
            if matches!(
                op,
                RenduOp::Comment {
                    is_directive: true,
                    ..
                }
            ) {
                None
            } else {
                Some((op, child))
            }
        })
    }
}
