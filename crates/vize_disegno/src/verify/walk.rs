//! The single verification walk over the owned S2 model.
//!
//! One pre-order traversal in page order (the order `print` emits lines),
//! carrying [`Rigor`] so the canonical-form checks share the walk instead
//! of costing a second one (the GHC anti-lesson: make each traversal do
//! more, not run more traversals). Per line the order is fixed —
//! span order (S2V001), then region nesting against the immediate owner
//! (S2V002), then the canonical form checks (S2V004–S2V006) — so an
//! aggregated report is deterministic and the TS-18 fixtures can pin it
//! whole.
//!
//! The keyword and span accessors match exhaustively with no `_` arm on
//! purpose: a new op variant must break this file loudly, the same
//! staleness discipline `folio/owned.rs` and the canary test enforce.

use alloc::vec::Vec;

use vize_carton::{Span, cstr};

use super::{Rigor, Violation, ViolationCode};
use crate::folio::{DisegnoFolio, FolioAttribute, FolioBinding, FolioIf, FolioOp};

/// The immediate owner of a nested line: its keyword and its span.
type Owner = Option<(&'static str, Span)>;

pub(super) fn walk(folio: &DisegnoFolio, rigor: Rigor, out: &mut Vec<Violation>) {
    for op in &folio.ops {
        visit_op(op, None, rigor, out);
    }
}

/// The keyword an op's folio line starts with — the owned twin of
/// `Op::mnemonic`.
fn keyword(op: &FolioOp) -> &'static str {
    match op {
        FolioOp::Element(_) => "ui.element",
        FolioOp::Component(_) => "ui.component",
        FolioOp::Text(_) => "ui.text",
        FolioOp::Interpolation(_) => "ui.interpolation",
        FolioOp::If(_) => "ui.if",
        FolioOp::For(_) => "ui.for",
        FolioOp::Slot(_) => "ui.slot",
    }
}

/// The op's own span, whichever variant carries it.
fn op_span(op: &FolioOp) -> Span {
    match op {
        FolioOp::Element(element) => element.span,
        FolioOp::Component(component) => component.span,
        FolioOp::Text(text) => text.span,
        FolioOp::Interpolation(interpolation) => interpolation.span,
        FolioOp::If(if_op) => if_op.span,
        FolioOp::For(for_op) => for_op.span,
        FolioOp::Slot(slot) => slot.span,
    }
}

/// The two structural checks every line gets, in their fixed order.
fn line_checks(kw: &'static str, span: Span, owner: Owner, out: &mut Vec<Violation>) {
    if span.start > span.end {
        out.push(Violation {
            code: ViolationCode::SpanOrder,
            span,
            message: cstr!("`{kw}` span runs backwards: {}:{}", span.start, span.end),
        });
    }
    if let Some((owner_kw, owner_span)) = owner
        && (span.start < owner_span.start || span.end > owner_span.end)
    {
        out.push(Violation {
            code: ViolationCode::RegionNesting,
            span,
            message: cstr!(
                "`{kw}` at {}:{} escapes its `{owner_kw}` owner {}:{}",
                span.start,
                span.end,
                owner_span.start,
                owner_span.end
            ),
        });
    }
}

fn visit_op(op: &FolioOp, owner: Owner, rigor: Rigor, out: &mut Vec<Violation>) {
    let kw = keyword(op);
    let span = op_span(op);
    line_checks(kw, span, owner, out);
    match op {
        FolioOp::Element(element) => body(
            kw,
            span,
            &element.attributes,
            &element.bindings,
            &element.children,
            rigor,
            out,
        ),
        FolioOp::Component(component) => body(
            kw,
            span,
            &component.attributes,
            &component.bindings,
            &component.children,
            rigor,
            out,
        ),
        FolioOp::Text(_) | FolioOp::Interpolation(_) => {}
        FolioOp::If(if_op) => visit_if(if_op, rigor, out),
        FolioOp::For(for_op) => {
            for child in &for_op.ops {
                visit_op(child, Some((kw, span)), rigor, out);
            }
        }
        FolioOp::Slot(slot) => body(
            kw,
            span,
            &slot.attributes,
            &slot.bindings,
            &slot.fallback,
            rigor,
            out,
        ),
    }
}

/// The grouped body an element or component owns: attributes, then
/// attached bindings (a `ui.model`'s own attributes nest under the model,
/// not the element), then children.
fn body(
    owner_kw: &'static str,
    owner_span: Span,
    attributes: &[FolioAttribute],
    bindings: &[FolioBinding],
    children: &[FolioOp],
    rigor: Rigor,
    out: &mut Vec<Violation>,
) {
    let owner = Some((owner_kw, owner_span));
    for attribute in attributes {
        line_checks("attr", attribute.span, owner, out);
    }
    for binding in bindings {
        match binding {
            FolioBinding::Bind(bind) => {
                line_checks("ui.bind", bind.span, owner, out);
            }
            FolioBinding::On(on) => {
                line_checks("ui.on", on.span, owner, out);
            }
            FolioBinding::Model(model) => {
                line_checks("ui.model", model.span, owner, out);
                for attribute in &model.attributes {
                    line_checks("attr", attribute.span, Some(("ui.model", model.span)), out);
                }
            }
            FolioBinding::SlotContent(content) => {
                line_checks("ui.slot-content", content.span, owner, out);
            }
            FolioBinding::VueDirective(directive) => {
                line_checks("vue.directive", directive.span, owner, out);
            }
            FolioBinding::VueCssBind(bind) => {
                line_checks("vue.css-bind", bind.span, owner, out);
            }
        }
    }
    for child in children {
        visit_op(child, owner, rigor, out);
    }
}

/// `ui.if`, with the canonical-form checks the type deliberately does not
/// encode (`op/control.rs`): at least one branch, a conditional leading
/// branch, unconditional only in trailing position.
fn visit_if(if_op: &FolioIf, rigor: Rigor, out: &mut Vec<Violation>) {
    let canonical = rigor == Rigor::Canonical;
    if canonical && if_op.branches.is_empty() {
        out.push(Violation {
            code: ViolationCode::EmptyIf,
            span: if_op.span,
            message: cstr!("`ui.if` owns no branch"),
        });
    }
    let owner = Some(("ui.if", if_op.span));
    let last = if_op.branches.len().saturating_sub(1);
    for (index, branch) in if_op.branches.iter().enumerate() {
        line_checks("branch", branch.span, owner, out);
        if canonical && branch.condition.is_none() {
            if index == 0 {
                out.push(Violation {
                    code: ViolationCode::LeadingElse,
                    span: branch.span,
                    message: cstr!("`ui.if` opens with an unconditional branch"),
                });
            } else if index != last {
                out.push(Violation {
                    code: ViolationCode::BranchOrder,
                    span: branch.span,
                    message: cstr!("unconditional branch before the trailing branch"),
                });
            }
        }
        for child in &branch.ops {
            visit_op(child, Some(("branch", branch.span)), rigor, out);
        }
    }
}
