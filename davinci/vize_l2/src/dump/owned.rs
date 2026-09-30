//! The owned mirror of the L2 op family, and the arena-to-owned
//! conversion.
//!
//! One mirror type per lifetime-carrying op type; the lifetime-free op
//! types ([`Span`], [`Namespace`], [`OpaqueReason`](crate::expr::OpaqueReason))
//! are reused directly, and the expression mirrors live in [`expr`].
//! [`Page::of`] is the bridge, and its matches are exhaustive
//! with no `_` arm on purpose: a new op variant must break this file
//! loudly (the same staleness discipline the canary test enforces).

use alloc::vec::Vec;

use vize_l0::{Span, String, ensure_sufficient_stack};

use crate::dump::Page;
use crate::op::{Attribute as IrAttribute, DynamicName, Namespace, Op as IrOp, Region};

mod binding;
mod expr;

pub use crate::dump::owned::binding::{
    Bind, Binding, Model, On, SlotContent, VueCloak, VueCssBind, VueDirective, VueHtml, VueMemo,
    VueOnce, VueShow, VueSlotScope, VueSync, VueText,
};
pub use crate::dump::owned::expr::{Contract, Expr, ForBinding};

use binding::own_binding;
use expr::own_expr;

/// Mirror of [`IrOp`]: one region op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// `ui.element`.
    Element(Element),
    /// `ui.component`.
    Component(Component),
    /// `ui.text`.
    Text(Text),
    /// `ui.interpolation`.
    Interpolation(Interpolation),
    /// `ui.comment`.
    Comment(Comment),
    /// `ui.if`.
    If(If),
    /// `ui.for`.
    For(For),
    /// `ui.slot`.
    Slot(Slot),
}

/// Mirror of [`DynamicName`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Name {
    /// A literal name.
    Static(String),
    /// A computed name.
    Dynamic(Expr),
}

/// Mirror of [`IrAttribute`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// IrAttribute name.
    pub name: String,
    /// The value; `None` for a bare boolean attribute.
    pub value: Option<String>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::ElementOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// Tag name.
    pub tag: String,
    /// Markup namespace.
    pub namespace: Namespace,
    /// Static attributes, in order.
    pub attributes: Vec<Attribute>,
    /// Attached bindings, in order.
    pub bindings: Vec<Binding>,
    /// The owned children region.
    pub children: Vec<Op>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::ComponentOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    /// Component name.
    pub name: String,
    /// Static attributes, in order.
    pub attributes: Vec<Attribute>,
    /// Attached bindings, in order.
    pub bindings: Vec<Binding>,
    /// The owned children region.
    pub children: Vec<Op>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::TextOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    /// The text content.
    pub content: String,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::InterpolationOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation {
    /// The rendered expression.
    pub expression: Expr,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::CommentOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// The comment body.
    pub content: String,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::IfOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    /// The branches, in order.
    pub branches: Vec<Branch>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::IfBranch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    /// The condition; `None` for the unconditional branch.
    pub condition: Option<Expr>,
    /// The branch's owned region.
    pub ops: Vec<Op>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::ForOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct For {
    /// The iteration binding.
    pub binding: ForBinding,
    /// The repeated region.
    pub ops: Vec<Op>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::SlotOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    /// The outlet name.
    pub name: Name,
    /// Static slot props, in order.
    pub attributes: Vec<Attribute>,
    /// Attached bindings, in order.
    pub bindings: Vec<Binding>,
    /// The fallback region.
    pub fallback: Vec<Op>,
    /// Source range.
    pub span: Span,
}

impl Page {
    /// Mirror a live arena tree into the owned document model.
    #[must_use]
    pub fn of(ops: &[IrOp<'_>]) -> Self {
        ensure_sufficient_stack(|| Self { ops: own_ops(ops) })
    }

    /// Total op count of the tree: region ops plus attached bindings, all
    /// levels. This is what the printed `ops=` header states.
    #[must_use]
    pub fn op_count(&self) -> u64 {
        ensure_sufficient_stack(|| self.ops.iter().map(count_op).sum())
    }
}

fn own_ops(ops: &[IrOp<'_>]) -> Vec<Op> {
    ops.iter()
        .map(|op| ensure_sufficient_stack(|| own_op(op)))
        .collect()
}

fn own_region(region: &Region<'_>) -> Vec<Op> {
    ensure_sufficient_stack(|| own_ops(&region.ops))
}

fn own_op(op: &IrOp<'_>) -> Op {
    ensure_sufficient_stack(|| own_op_guarded(op))
}

fn own_op_guarded(op: &IrOp<'_>) -> Op {
    match op {
        IrOp::Element(element) => Op::Element(Element {
            tag: String::from(element.tag),
            namespace: element.namespace,
            attributes: element.attributes.iter().map(own_attribute).collect(),
            bindings: element.bindings.iter().map(own_binding).collect(),
            children: own_region(&element.children),
            span: element.span,
        }),
        IrOp::Component(component) => Op::Component(Component {
            name: String::from(component.name),
            attributes: component.attributes.iter().map(own_attribute).collect(),
            bindings: component.bindings.iter().map(own_binding).collect(),
            children: own_region(&component.children),
            span: component.span,
        }),
        IrOp::Text(text) => Op::Text(Text {
            content: String::from(text.content),
            span: text.span,
        }),
        IrOp::Interpolation(interpolation) => Op::Interpolation(Interpolation {
            expression: own_expr(&interpolation.expression),
            span: interpolation.span,
        }),
        IrOp::Comment(comment) => Op::Comment(Comment {
            content: String::from(comment.content),
            span: comment.span,
        }),
        IrOp::If(if_op) => Op::If(If {
            branches: if_op
                .branches
                .iter()
                .map(|branch| Branch {
                    condition: branch.condition.as_ref().map(own_expr),
                    ops: own_region(&branch.region),
                    span: branch.span,
                })
                .collect(),
            span: if_op.span,
        }),
        IrOp::For(for_op) => Op::For(For {
            binding: ForBinding {
                source: own_expr(&for_op.binding.source),
                value: own_expr(&for_op.binding.value),
                key: for_op.binding.key.as_ref().map(own_expr),
                index: for_op.binding.index.as_ref().map(own_expr),
            },
            ops: own_region(&for_op.region),
            span: for_op.span,
        }),
        IrOp::Slot(slot) => Op::Slot(Slot {
            name: own_name(&slot.name),
            attributes: slot.attributes.iter().map(own_attribute).collect(),
            bindings: slot.bindings.iter().map(own_binding).collect(),
            fallback: own_region(&slot.fallback),
            span: slot.span,
        }),
    }
}

pub(super) fn own_name(name: &DynamicName<'_>) -> Name {
    match name {
        DynamicName::Static(text) => Name::Static(String::from(*text)),
        DynamicName::Dynamic(expr) => Name::Dynamic(own_expr(expr)),
    }
}

pub(super) fn own_attribute(attribute: &IrAttribute<'_>) -> Attribute {
    Attribute {
        name: String::from(attribute.name),
        value: attribute.value.map(String::from),
        span: attribute.span,
    }
}

fn count_op(op: &Op) -> u64 {
    ensure_sufficient_stack(|| count_op_guarded(op))
}

fn count_op_guarded(op: &Op) -> u64 {
    match op {
        Op::Element(element) => {
            1 + element.bindings.len() as u64 + element.children.iter().map(count_op).sum::<u64>()
        }
        Op::Component(component) => {
            1 + component.bindings.len() as u64
                + component.children.iter().map(count_op).sum::<u64>()
        }
        Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => 1,
        Op::If(if_op) => {
            1 + if_op
                .branches
                .iter()
                .flat_map(|branch| branch.ops.iter())
                .map(count_op)
                .sum::<u64>()
        }
        Op::For(for_op) => 1 + for_op.ops.iter().map(count_op).sum::<u64>(),
        Op::Slot(slot) => {
            1 + slot.bindings.len() as u64 + slot.fallback.iter().map(count_op).sum::<u64>()
        }
    }
}
