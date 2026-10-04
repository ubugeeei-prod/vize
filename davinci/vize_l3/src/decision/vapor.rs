//! Static Vapor semantics collected during the sole canonical decision walk.
//!
//! Ordered parts borrow the original L2 nodes. L4 owns HTML/JavaScript spelling;
//! these facts decide eligibility and original root boundaries, never strings.

use alloc::vec::Vec;
use core::ops::Range;
use vize_l0::{Span, id::NodeId};
use vize_l2::op::{CommentOp, ElementOp, InterpolationOp, TextOp};

pub(super) mod build;
mod expression;
mod file;
mod setup;
mod template;
#[cfg(test)]
mod tests;
pub use expression::{VaporExpression, VaporValueKind};
pub use file::{NativeVaporFileAnalysis, build_vapor_file_decisions};
pub use setup::{NativeSelectedSetupVaporAnalysis, build_native_selected_setup_vapor_decisions};
pub use template::{NativeTemplateVaporAnalysis, build_native_vapor_file_decisions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaporUnsupported {
    Operation,
    Namespace,
    ElementSemantics,
    VoidChildren,
    AttributeSemantics,
    DuplicateAttribute,
    TextNormalization,
    RootTextMarkup,
    UnsafeComment,
    Binding,
    ExpressionOrigin,
    Expression,
    SetupRead,
    StringNormalization,
    NestedInterpolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaporRejection {
    pub node: NodeId,
    pub span: Span,
    pub reason: VaporUnsupported,
}

/// One original opening, closing or leaf event, in canonical source order.
#[derive(Debug, Clone, Copy)]
pub enum VaporPart<'owner, 'arena> {
    Open {
        node: NodeId,
        element: &'owner ElementOp<'arena>,
        void: bool,
    },
    Close {
        node: NodeId,
        element: &'owner ElementOp<'arena>,
    },
    Text {
        node: NodeId,
        text: &'owner TextOp<'arena>,
    },
    Interpolation {
        node: NodeId,
        interpolation: &'owner InterpolationOp<'arena>,
        expression: VaporExpression<'owner, 'arena>,
    },
    Comment {
        node: NodeId,
        comment: &'owner CommentOp<'arena>,
    },
}

/// A real root's complete contiguous part range; callers cannot mint one.
#[derive(Debug)]
pub struct VaporRoot {
    pub(super) ordinal: usize,
    pub(super) node: NodeId,
    pub(super) span: Span,
    pub(super) parts: Range<usize>,
}

impl VaporRoot {
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

/// Borrowed target facts attached to the exact immutable native analysis.
#[derive(Debug)]
pub struct VaporFacts<'owner, 'arena> {
    pub(super) parts: Vec<VaporPart<'owner, 'arena>>,
    pub(super) roots: Vec<VaporRoot>,
    pub(super) unsupported: Vec<VaporRejection>,
    pub(super) inherit_attrs: Option<NodeId>,
}

impl<'owner, 'arena> VaporFacts<'owner, 'arena> {
    #[must_use]
    pub fn roots(&self) -> &[VaporRoot] {
        &self.roots
    }
    #[must_use]
    pub fn parts(&self, root: &VaporRoot) -> Option<&[VaporPart<'owner, 'arena>]> {
        self.roots
            .get(root.ordinal)
            .is_some_and(|original| core::ptr::eq(original, root))
            .then(|| self.parts.get(root.parts.clone()))
            .flatten()
    }
    #[must_use]
    pub fn unsupported(&self) -> &[VaporRejection] {
        &self.unsupported
    }
    /// Only one non-comment native root receives root-template fallthrough.
    #[must_use]
    pub const fn inherit_attrs(&self) -> Option<NodeId> {
        self.inherit_attrs
    }
}
