//! Vue JSX decisions beside the sole original owning L2 File.
//!
//! This projection accepts static structure, resolved component names and scalar
//! expression-container children from original File records.
//! It does not grant script/module emission, entity decoding, slots or runtime
//! completion. The producer visits the existing flat records, never the AST.

use alloc::{boxed::Box, vec::Vec};
use vize_l0::Span;
use vize_l2::lang::js::{JsxFile, JsxNode};
use vize_l2::resolution::BindingId;

mod build;
pub use build::build_jsx_decisions;

/// A target-neutral role derived from this owner's same-walk syntax record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxDecisionKind<'a> {
    Root,
    Element,
    Opening {
        self_closing: bool,
    },
    Closing,
    Intrinsic(&'a str),
    Component(&'a str),
    Member,
    Property(&'a str),
    StaticAttribute {
        name: &'a str,
        value: Option<&'a str>,
    },
    AttributeName(&'a str),
    AttributeString(&'a str),
    /// Original parser value; JSX whitespace/entities still need target rules.
    Text(&'a str),
    EmptyContainer,
    ExpressionContainer,
    Read {
        name: &'a str,
        binding: BindingId,
    },
    /// Exact original numeric-literal bits; target formatting is still separate.
    Number(u64),
    String(&'a str),
    Boolean(bool),
    Null,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxIssueKind {
    InvalidRecord,
    Expression,
    Fragment,
    DirectiveOrSlot,
    SpreadAttribute,
    SpreadChild,
    Attribute,
    UnresolvedComponent,
}

/// Diagnostic location in the retained original File, never an admission key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsxIssue {
    pub kind: JsxIssueKind,
    pub span: Span,
}

/// Decisions consume and retain the opaque original syntax/File owner.
/// A caller cannot associate independent tables or foreign numeric ids.
///
/// ```compile_fail
/// use vize_l2::lang::js::JsxFile;
/// use vize_l3::jsx::NativeJsxAnalysis;
/// fn forge<'a>(owner: JsxFile<'a>) {
///     let _ = NativeJsxAnalysis { owner, decisions: vec![] };
/// }
/// ```
pub struct NativeJsxAnalysis<'a> {
    owner: JsxFile<'a>,
    decisions: Vec<JsxDecisionKind<'a>>,
}

impl<'a> NativeJsxAnalysis<'a> {
    #[must_use]
    pub fn owner(&self) -> &JsxFile<'a> {
        &self.owner
    }

    /// Query one existing row only after checking the actual retained owner.
    #[must_use]
    pub fn decision_for<'f>(&'f self, node: JsxNode<'f, 'a>) -> Option<JsxDecision<'f, 'a>> {
        if !core::ptr::eq(node.owner(), &self.owner) {
            return None;
        }
        Some(JsxDecision {
            node,
            kind: *self.decisions.get(node.index())?,
        })
    }

    pub fn decisions(&self) -> impl Iterator<Item = JsxDecision<'_, 'a>> {
        self.decisions
            .iter()
            .enumerate()
            .filter_map(|(index, kind)| {
                Some(JsxDecision {
                    node: self.owner.node(index)?,
                    kind: *kind,
                })
            })
    }
}

/// A completed decision borrows its exact node and original owner together.
///
/// ```compile_fail
/// use vize_l3::jsx::{JsxDecision, NativeJsxAnalysis};
/// fn discard<'a>(analysis: NativeJsxAnalysis<'a>) {
///     let decision = analysis.decisions().next().unwrap();
///     drop(analysis);
///     let _ = decision.node();
/// }
/// ```
/// Public role values cannot mint a decision for an arbitrary foreign node:
/// ```compile_fail
/// use vize_l2::lang::js::JsxNode;
/// use vize_l3::jsx::{JsxDecision, JsxDecisionKind};
/// fn forge<'f, 'a>(node: JsxNode<'f, 'a>) {
///     let _ = JsxDecision { node, kind: JsxDecisionKind::Root };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct JsxDecision<'owner, 'a> {
    node: JsxNode<'owner, 'a>,
    kind: JsxDecisionKind<'a>,
}

impl<'owner, 'a> JsxDecision<'owner, 'a> {
    #[must_use]
    pub fn node(self) -> JsxNode<'owner, 'a> {
        self.node
    }
    #[must_use]
    pub fn kind(self) -> JsxDecisionKind<'a> {
        self.kind
    }
}

/// A refusal keeps every original owner and completed lower fact intact.
/// It exposes no partially admitted decision iterator or mutable AST/File.
pub struct RejectedJsxAnalysis<'a> {
    owner: JsxFile<'a>,
    issues: Vec<JsxIssue>,
}

impl<'a> RejectedJsxAnalysis<'a> {
    #[must_use]
    pub fn owner(&self) -> &JsxFile<'a> {
        &self.owner
    }
    #[must_use]
    pub fn issues(&self) -> &[JsxIssue] {
        &self.issues
    }
}

type BuildResult<'a> = Result<NativeJsxAnalysis<'a>, Box<RejectedJsxAnalysis<'a>>>;
