//! Bounded server-rendering semantics from the sole canonical decision walk.
//!
//! These facts retain the actual L2 nodes. HTML eligibility, void content,
//! root attribute inheritance and fragment boundaries are SSR decisions;
//! string escaping, helper names and JavaScript spelling belong to L4.

use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId};
use vize_l2::op::{CommentOp, ElementOp, TextOp};

pub(super) mod build;
mod file;
mod native;
#[cfg(test)]
mod tests;
pub use file::{NativeSsrFileAnalysis, build_ssr_file_decisions};
pub use native::{NativeSsrBuildError, NativeTemplateSsrAnalysis, build_native_ssr_file_decisions};

/// A whole SSR view retains unsupported locations instead of partial output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsrUnsupported {
    Operation,
    Namespace,
    ElementName,
    ElementSemantics,
    VoidChildren,
    AttributeName,
    AttributeSemantics,
    DuplicateAttribute,
    Binding,
    UnsafeComment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsrRejection {
    pub node: NodeId,
    pub span: Span,
    pub reason: SsrUnsupported,
}

/// The original ordered node at an admitted opening, leaf or closing event.
#[derive(Debug, Clone, Copy)]
pub enum SsrPart<'owner, 'arena> {
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
    Comment {
        node: NodeId,
        comment: &'owner CommentOp<'arena>,
    },
}

/// Immutable SSR facts beside their exact sealed native analysis.
#[derive(Debug)]
pub struct SsrFacts<'owner, 'arena> {
    parts: Vec<SsrPart<'owner, 'arena>>,
    unsupported: Vec<SsrRejection>,
    inherit_attrs: Option<NodeId>,
    fragment: bool,
}

impl<'owner, 'arena> SsrFacts<'owner, 'arena> {
    #[must_use]
    pub fn parts(&self) -> &[SsrPart<'owner, 'arena>] {
        &self.parts
    }

    #[must_use]
    pub fn unsupported(&self) -> &[SsrRejection] {
        &self.unsupported
    }

    /// A single non-comment native root receives the component's fallthrough.
    #[must_use]
    pub const fn inherit_attrs(&self) -> Option<NodeId> {
        self.inherit_attrs
    }

    /// Multiple authored root nodes need SSR hydration fragment boundaries.
    #[must_use]
    pub const fn fragment(&self) -> bool {
        self.fragment
    }
}
