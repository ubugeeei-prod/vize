//! The owned mirror of the S2 op family, and the arena-to-owned
//! conversion.
//!
//! One mirror type per lifetime-carrying op type; the lifetime-free op
//! types ([`Span`], [`Namespace`], [`OpaqueReason`](crate::expr::OpaqueReason))
//! are reused directly, and the expression mirrors live in [`expr`].
//! [`S2Folio::of`] is the bridge, and its matches are exhaustive
//! with no `_` arm on purpose: a new op variant must break this file
//! loudly (the same staleness discipline the canary test enforces).

use alloc::vec::Vec;

use vize_s0::{Span, String, ensure_sufficient_stack};

use super::S2Folio;
use crate::op::{Attribute, DynamicName, Namespace, Op, Region};

mod binding;
mod expr;

pub use binding::{
    FolioBind, FolioBinding, FolioModel, FolioOn, FolioSlotContent, FolioVueCloak, FolioVueCssBind,
    FolioVueDirective, FolioVueHtml, FolioVueMemo, FolioVueOnce, FolioVueShow, FolioVueSlotScope,
    FolioVueSync, FolioVueText,
};
pub use expr::{FolioContract, FolioExpr, FolioForBinding};

use binding::own_binding;
use expr::own_expr;

/// Mirror of [`Op`]: one region op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolioOp {
    /// `ui.element`.
    Element(FolioElement),
    /// `ui.component`.
    Component(FolioComponent),
    /// `ui.text`.
    Text(FolioText),
    /// `ui.interpolation`.
    Interpolation(FolioInterpolation),
    /// `ui.if`.
    If(FolioIf),
    /// `ui.for`.
    For(FolioFor),
    /// `ui.slot`.
    Slot(FolioSlot),
}

/// Mirror of [`DynamicName`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolioName {
    /// A literal name.
    Static(String),
    /// A computed name.
    Dynamic(FolioExpr),
}

/// Mirror of [`Attribute`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioAttribute {
    /// Attribute name.
    pub name: String,
    /// The value; `None` for a bare boolean attribute.
    pub value: Option<String>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::ElementOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioElement {
    /// Tag name.
    pub tag: String,
    /// Markup namespace.
    pub namespace: Namespace,
    /// Static attributes, in order.
    pub attributes: Vec<FolioAttribute>,
    /// Attached bindings, in order.
    pub bindings: Vec<FolioBinding>,
    /// The owned children region.
    pub children: Vec<FolioOp>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::ComponentOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioComponent {
    /// Component name.
    pub name: String,
    /// Static attributes, in order.
    pub attributes: Vec<FolioAttribute>,
    /// Attached bindings, in order.
    pub bindings: Vec<FolioBinding>,
    /// The owned children region.
    pub children: Vec<FolioOp>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::TextOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioText {
    /// The text content.
    pub content: String,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::InterpolationOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioInterpolation {
    /// The rendered expression.
    pub expression: FolioExpr,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::IfOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioIf {
    /// The branches, in order.
    pub branches: Vec<FolioBranch>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::IfBranch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioBranch {
    /// The condition; `None` for the unconditional branch.
    pub condition: Option<FolioExpr>,
    /// The branch's owned region.
    pub ops: Vec<FolioOp>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::ForOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioFor {
    /// The iteration binding.
    pub binding: FolioForBinding,
    /// The repeated region.
    pub ops: Vec<FolioOp>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::SlotOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioSlot {
    /// The outlet name.
    pub name: FolioName,
    /// Static slot props, in order.
    pub attributes: Vec<FolioAttribute>,
    /// Attached bindings, in order.
    pub bindings: Vec<FolioBinding>,
    /// The fallback region.
    pub fallback: Vec<FolioOp>,
    /// Source range.
    pub span: Span,
}

impl S2Folio {
    /// Mirror a live arena tree into the owned document model.
    #[must_use]
    pub fn of(ops: &[Op<'_>]) -> Self {
        ensure_sufficient_stack(|| Self { ops: own_ops(ops) })
    }

    /// Total op count of the tree: region ops plus attached bindings, all
    /// levels. This is what the printed `ops=` header states.
    #[must_use]
    pub fn op_count(&self) -> u64 {
        ensure_sufficient_stack(|| self.ops.iter().map(count_op).sum())
    }
}

fn own_ops(ops: &[Op<'_>]) -> Vec<FolioOp> {
    ops.iter()
        .map(|op| ensure_sufficient_stack(|| own_op(op)))
        .collect()
}

fn own_region(region: &Region<'_>) -> Vec<FolioOp> {
    ensure_sufficient_stack(|| own_ops(&region.ops))
}

fn own_op(op: &Op<'_>) -> FolioOp {
    ensure_sufficient_stack(|| own_op_guarded(op))
}

fn own_op_guarded(op: &Op<'_>) -> FolioOp {
    match op {
        Op::Element(element) => FolioOp::Element(FolioElement {
            tag: String::from(element.tag),
            namespace: element.namespace,
            attributes: element.attributes.iter().map(own_attribute).collect(),
            bindings: element.bindings.iter().map(own_binding).collect(),
            children: own_region(&element.children),
            span: element.span,
        }),
        Op::Component(component) => FolioOp::Component(FolioComponent {
            name: String::from(component.name),
            attributes: component.attributes.iter().map(own_attribute).collect(),
            bindings: component.bindings.iter().map(own_binding).collect(),
            children: own_region(&component.children),
            span: component.span,
        }),
        Op::Text(text) => FolioOp::Text(FolioText {
            content: String::from(text.content),
            span: text.span,
        }),
        Op::Interpolation(interpolation) => FolioOp::Interpolation(FolioInterpolation {
            expression: own_expr(&interpolation.expression),
            span: interpolation.span,
        }),
        Op::If(if_op) => FolioOp::If(FolioIf {
            branches: if_op
                .branches
                .iter()
                .map(|branch| FolioBranch {
                    condition: branch.condition.as_ref().map(own_expr),
                    ops: own_region(&branch.region),
                    span: branch.span,
                })
                .collect(),
            span: if_op.span,
        }),
        Op::For(for_op) => FolioOp::For(FolioFor {
            binding: FolioForBinding {
                source: own_expr(&for_op.binding.source),
                value: own_expr(&for_op.binding.value),
                key: for_op.binding.key.as_ref().map(own_expr),
                index: for_op.binding.index.as_ref().map(own_expr),
            },
            ops: own_region(&for_op.region),
            span: for_op.span,
        }),
        Op::Slot(slot) => FolioOp::Slot(FolioSlot {
            name: own_name(&slot.name),
            attributes: slot.attributes.iter().map(own_attribute).collect(),
            bindings: slot.bindings.iter().map(own_binding).collect(),
            fallback: own_region(&slot.fallback),
            span: slot.span,
        }),
    }
}

pub(super) fn own_name(name: &DynamicName<'_>) -> FolioName {
    match name {
        DynamicName::Static(text) => FolioName::Static(String::from(*text)),
        DynamicName::Dynamic(expr) => FolioName::Dynamic(own_expr(expr)),
    }
}

pub(super) fn own_attribute(attribute: &Attribute<'_>) -> FolioAttribute {
    FolioAttribute {
        name: String::from(attribute.name),
        value: attribute.value.map(String::from),
        span: attribute.span,
    }
}

fn count_op(op: &FolioOp) -> u64 {
    ensure_sufficient_stack(|| count_op_guarded(op))
}

fn count_op_guarded(op: &FolioOp) -> u64 {
    match op {
        FolioOp::Element(element) => {
            1 + element.bindings.len() as u64 + element.children.iter().map(count_op).sum::<u64>()
        }
        FolioOp::Component(component) => {
            1 + component.bindings.len() as u64
                + component.children.iter().map(count_op).sum::<u64>()
        }
        FolioOp::Text(_) | FolioOp::Interpolation(_) => 1,
        FolioOp::If(if_op) => {
            1 + if_op
                .branches
                .iter()
                .flat_map(|branch| branch.ops.iter())
                .map(count_op)
                .sum::<u64>()
        }
        FolioOp::For(for_op) => 1 + for_op.ops.iter().map(count_op).sum::<u64>(),
        FolioOp::Slot(slot) => {
            1 + slot.bindings.len() as u64 + slot.fallback.iter().map(count_op).sum::<u64>()
        }
    }
}
