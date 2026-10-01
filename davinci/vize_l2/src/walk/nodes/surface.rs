//! Exhaustive borrowed node accessors, shared by producers and consumers.

use vize_l0::Span;

use super::NodeRef;
use crate::expr::ExprRef;
use crate::op::{Attribute, BindingOp, DynamicName, Op};

impl<'s, 'a> NodeRef<'s, 'a> {
    /// The node's stage-wide mnemonic.
    #[must_use]
    pub fn mnemonic(self) -> &'static str {
        match self {
            Self::Op(op) => op.mnemonic(),
            Self::Binding(binding) => binding.mnemonic(),
        }
    }

    /// The file-absolute authored span.
    #[must_use]
    pub fn span(self) -> Span {
        match self {
            Self::Op(op) => match op {
                Op::Element(it) => it.span,
                Op::Component(it) => it.span,
                Op::Text(it) => it.span,
                Op::Interpolation(it) => it.span,
                Op::Comment(it) => it.span,
                Op::If(it) => it.span,
                Op::For(it) => it.span,
                Op::Slot(it) => it.span,
            },
            Self::Binding(binding) => match binding {
                BindingOp::Bind(it) => it.span,
                BindingOp::On(it) => it.span,
                BindingOp::Model(it) => it.span,
                BindingOp::SlotContent(it) => it.span,
                BindingOp::VueDirective(it) => it.span,
                BindingOp::VueCssBind(it) => it.span,
                BindingOp::VueSync(it) => it.span,
                BindingOp::VueSlotScope(it) => it.span,
                BindingOp::VueOnce(it) => it.span,
                BindingOp::VueMemo(it) => it.span,
                BindingOp::VueShow(it) => it.span,
                BindingOp::VueHtml(it) => it.span,
                BindingOp::VueText(it) => it.span,
                BindingOp::VueCloak(it) => it.span,
            },
        }
    }

    /// Attributes owned by this node, without allocating a mirror.
    #[must_use]
    pub fn attributes(self) -> &'s [Attribute<'a>] {
        match self {
            Self::Op(Op::Element(it)) => &it.attributes,
            Self::Op(Op::Component(it)) => &it.attributes,
            Self::Op(Op::Slot(it)) => &it.attributes,
            Self::Binding(BindingOp::Model(it)) => &it.attributes,
            Self::Op(
                Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) | Op::If(_) | Op::For(_),
            )
            | Self::Binding(
                BindingOp::Bind(_)
                | BindingOp::On(_)
                | BindingOp::SlotContent(_)
                | BindingOp::VueDirective(_)
                | BindingOp::VueCssBind(_)
                | BindingOp::VueSync(_)
                | BindingOp::VueSlotScope(_)
                | BindingOp::VueOnce(_)
                | BindingOp::VueMemo(_)
                | BindingOp::VueShow(_)
                | BindingOp::VueHtml(_)
                | BindingOp::VueText(_)
                | BindingOp::VueCloak(_),
            ) => &[],
        }
    }

    /// Visit each expression position on this node without reparsing.
    ///
    /// If conditions belong to the `ui.if` node. A filter chain remains
    /// one dialect expression; its base/segments are that payload's API.
    /// Model read/write remain distinct positions even when they share
    /// the same payload pointer.
    pub fn for_each_expression(self, visit: &mut impl FnMut(ExprRef<'a>)) {
        match self {
            Self::Op(op) => match op {
                Op::Element(_) | Op::Component(_) | Op::Text(_) | Op::Comment(_) => {}
                Op::Interpolation(it) => visit(it.expression),
                Op::If(it) => {
                    for branch in &it.branches {
                        optional(branch.condition, visit);
                    }
                }
                Op::For(it) => {
                    visit(it.binding.source);
                    visit(it.binding.value);
                    optional(it.binding.key, visit);
                    optional(it.binding.index, visit);
                }
                Op::Slot(it) => name(Some(it.name), visit),
            },
            Self::Binding(binding) => match binding {
                BindingOp::Bind(it) => {
                    name(it.name, visit);
                    optional(it.value, visit);
                }
                BindingOp::On(it) => {
                    name(it.name, visit);
                    optional(it.handler, visit);
                }
                BindingOp::Model(it) => {
                    name(it.argument, visit);
                    visit(it.contract.read);
                    visit(it.contract.write);
                }
                BindingOp::SlotContent(it) => {
                    name(it.name, visit);
                    optional(it.params, visit);
                }
                BindingOp::VueDirective(it) => {
                    name(it.argument, visit);
                    optional(it.value, visit);
                }
                BindingOp::VueCssBind(it) => visit(it.value),
                BindingOp::VueSync(it) => visit(it.value),
                BindingOp::VueSlotScope(it) => optional(it.params, visit),
                BindingOp::VueOnce(_) | BindingOp::VueCloak(_) => {}
                BindingOp::VueMemo(it) => visit(it.value),
                BindingOp::VueShow(it) => visit(it.value),
                BindingOp::VueHtml(it) => optional(it.value, visit),
                BindingOp::VueText(it) => optional(it.value, visit),
            },
        }
    }
}

fn optional<'a>(expr: Option<ExprRef<'a>>, visit: &mut impl FnMut(ExprRef<'a>)) {
    if let Some(expr) = expr {
        visit(expr);
    }
}

fn name<'a>(name: Option<DynamicName<'a>>, visit: &mut impl FnMut(ExprRef<'a>)) {
    if let Some(DynamicName::Dynamic(expr)) = name {
        visit(expr);
    }
}
