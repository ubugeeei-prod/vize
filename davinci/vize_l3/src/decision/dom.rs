//! Bounded DOM semantic facts beside their exact canonical L2 owner.
//!
//! These are eligibility and authored-order facts. Vue patch bits, helper
//! spellings, dynamic property strings and block-tree syntax belong to L4.

use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId, side_table::SideTable};
use vize_l2::{
    expr::JsExpr,
    op::{BindingOp, Op},
};

pub(super) mod build;
mod context;
pub mod control;
mod dependencies;

pub use context::ContextOnly;
pub use dependencies::DomDependency;

/// A complete expression's references all have explicitly declared context bindings.
///
/// Implementations must check the retained AST's identity against a complete
/// native resolution table, require a nonempty occurrence set and check every
/// reference against the context registry. Zero-reference nonliterals do not
/// earn runtime or constant eligibility from the absence of occurrences.
/// This supplies binding semantics, never target patch bits or guessed purity.
pub trait DomExpressionFacts {
    fn is_context_only(&self, expression: &JsExpr<'_>) -> bool;
}

/// Default admission: only retained literal ASTs have supported semantics.
pub struct LiteralExpressions;

impl DomExpressionFacts for LiteralExpressions {
    fn is_context_only(&self, _: &JsExpr<'_>) -> bool {
        false
    }
}

/// Semantic reason a bounded DOM producer cannot admit this authored surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomUnsupported {
    Operation,
    Namespace,
    SpecialAttribute,
    Binding,
    BindingName,
    BindingModifiers,
    MissingValue,
    DuplicateProperty,
    Expression,
    ConditionalShape,
    ConditionalCondition,
    ConditionalRoot,
}

/// Unsupported semantics retain their actual authored location from the same walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomRejection {
    pub node: NodeId,
    pub span: Span,
    pub reason: DomUnsupported,
}

/// Root representation eligibility, before runtime encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomRootKind {
    Empty,
    Direct,
    Fragment { single_non_comment: bool },
}

/// An authored contiguous text/interpolation group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomText {
    pub nodes: Vec<NodeId>,
    pub dynamic: bool,
}

/// Ordered child groups, without a second tree or synthetic node ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomChild {
    Text(DomText),
    Node(NodeId),
}

/// Eligibility for direct text children versus a vnode child sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomChildren {
    Empty,
    Text(DomText),
    Array(Vec<DomChild>),
}

/// Which authored values can change under the admitted expression policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DomChanges {
    pub text: bool,
    pub class: bool,
    pub style: bool,
    pub properties: bool,
}

/// Semantic role of an admitted static property name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyRole {
    Property,
    Class,
    Style,
}

/// Value semantics proved by the retained AST or complete context registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    LiteralConstant,
    ContextDependent,
}

/// One exact canonical region op and its DOM eligibility.
#[derive(Debug)]
pub struct DomNode<'owner, 'arena> {
    pub(super) op: &'owner Op<'arena>,
    pub block_eligible: bool,
    pub children: DomChildren,
    pub changes: DomChanges,
    pub dynamic_property_bindings: Vec<NodeId>,
}

impl<'owner, 'arena> DomNode<'owner, 'arena> {
    #[must_use]
    pub fn op(&self) -> &'owner Op<'arena> {
        self.op
    }
}

/// One exact canonical binding and its semantic role/value classification.
#[derive(Debug)]
pub struct DomBinding<'owner, 'arena> {
    pub(super) binding: &'owner BindingOp<'arena>,
    pub role: PropertyRole,
    pub value: ValueKind,
}

impl<'owner, 'arena> DomBinding<'owner, 'arena> {
    #[must_use]
    pub fn binding(&self) -> &'owner BindingOp<'arena> {
        self.binding
    }
}

/// Ordered root structure, chosen in the existing native decision walk.
#[derive(Debug)]
pub struct DomRoot {
    pub kind: DomRootKind,
    pub children: Vec<DomChild>,
}

/// Read-only semantic facts tied to the actual immutable L2 owner's borrow.
#[derive(Debug)]
pub struct DomFacts<'owner, 'arena> {
    pub(super) root: DomRoot,
    pub(super) nodes: SideTable<DomNode<'owner, 'arena>>,
    pub(super) bindings: SideTable<DomBinding<'owner, 'arena>>,
    pub(super) controls: SideTable<control::DomConditional<'owner, 'arena>>,
    pub(super) dependencies: Vec<DomDependency>,
    pub(super) unsupported: Vec<DomRejection>,
}

impl<'owner, 'arena> DomFacts<'owner, 'arena> {
    #[must_use]
    pub fn root(&self) -> &DomRoot {
        &self.root
    }

    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&DomNode<'owner, 'arena>> {
        self.nodes.get(id)
    }

    #[must_use]
    pub fn binding(&self, id: NodeId) -> Option<&DomBinding<'owner, 'arena>> {
        self.bindings.get(id)
    }

    /// Checked conditional ownership and ordered direct branch roots.
    #[must_use]
    pub fn conditional(&self, id: NodeId) -> Option<&control::DomConditional<'owner, 'arena>> {
        self.controls.get(id)
    }

    /// Semantic first-use demands, with no runtime names or numeric flags.
    #[must_use]
    pub fn dependencies(&self) -> &[DomDependency] {
        &self.dependencies
    }

    #[must_use]
    pub fn unsupported(&self) -> &[DomRejection] {
        &self.unsupported
    }
}
