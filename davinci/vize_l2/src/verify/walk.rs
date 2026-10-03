//! The single verification walk over the owned L2 model.
//!
//! One pre-order traversal in page order (the order `print` emits lines),
//! carrying [`Rigor`] so the canonical-form checks share the walk instead
//! of costing a second one (the GHC anti-lesson: make each traversal do
//! more, not run more traversals). Per line the order is fixed —
//! span order (L2V001), then region nesting against the immediate owner
//! (L2V002), then the canonical form checks (L2V004–L2V006) — so an
//! aggregated report is deterministic and the TS-18 fixtures can pin it
//! whole.
//!
//! The keyword and span accessors match exhaustively with no `_` arm on
//! purpose: a new op variant must break this file loudly, the same
//! staleness discipline `folio/owned.rs` and the canary test enforce.

use alloc::vec::Vec;

use vize_l0::{Span, cstr};

use super::{Rigor, Violation, ViolationCode};
use crate::dump::{
    Attribute as DumpAttribute, Binding as DumpBinding, If as DumpIf, Op as DumpOp, Page as L2Page,
};

/// The immediate owner of a nested line: its keyword and its span.
type Owner = Option<(&'static str, Span)>;

pub(super) fn walk(folio: &L2Page, rigor: Rigor, out: &mut Vec<Violation>) {
    for op in &folio.ops {
        visit_op(op, None, rigor, out);
    }
}

/// The keyword an op's folio line starts with — the owned twin of
/// `Op::mnemonic`.
fn keyword(op: &DumpOp) -> &'static str {
    match op {
        DumpOp::Element(_) => "ui.element",
        DumpOp::Component(_) => "ui.component",
        DumpOp::Text(_) => "ui.text",
        DumpOp::Interpolation(_) => "ui.interpolation",
        DumpOp::Comment(_) => "ui.comment",
        DumpOp::If(_) => "ui.if",
        DumpOp::For(_) | DumpOp::OriginalFor(_) => "ui.for",
        DumpOp::Slot(_) => "ui.slot",
    }
}

/// The op's own span, whichever variant carries it.
fn op_span(op: &DumpOp) -> Span {
    match op {
        DumpOp::Element(element) => element.span,
        DumpOp::Component(component) => component.span,
        DumpOp::Text(text) => text.span,
        DumpOp::Interpolation(interpolation) => interpolation.span,
        DumpOp::Comment(comment) => comment.span,
        DumpOp::If(if_op) => if_op.span,
        DumpOp::For(for_op) => for_op.span,
        DumpOp::OriginalFor(for_op) => for_op.span,
        DumpOp::Slot(slot) => slot.span,
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

fn visit_op(op: &DumpOp, owner: Owner, rigor: Rigor, out: &mut Vec<Violation>) {
    let kw = keyword(op);
    let span = op_span(op);
    line_checks(kw, span, owner, out);
    match op {
        DumpOp::Element(element) => body(
            kw,
            span,
            &element.attributes,
            &element.bindings,
            &element.children,
            rigor,
            out,
        ),
        DumpOp::Component(component) => body(
            kw,
            span,
            &component.attributes,
            &component.bindings,
            &component.children,
            rigor,
            out,
        ),
        DumpOp::Text(_) | DumpOp::Interpolation(_) | DumpOp::Comment(_) => {}
        DumpOp::If(if_op) => visit_if(if_op, rigor, out),
        DumpOp::For(for_op) => {
            for child in &for_op.ops {
                visit_op(child, Some((kw, span)), rigor, out);
            }
        }
        DumpOp::OriginalFor(for_op) => {
            for child in &for_op.ops {
                visit_op(child, Some((kw, span)), rigor, out);
            }
        }
        DumpOp::Slot(slot) => body(
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
    attributes: &[DumpAttribute],
    bindings: &[DumpBinding],
    children: &[DumpOp],
    rigor: Rigor,
    out: &mut Vec<Violation>,
) {
    let owner = Some((owner_kw, owner_span));
    for attribute in attributes {
        line_checks("attr", attribute.span, owner, out);
    }
    for binding in bindings {
        match binding {
            DumpBinding::Bind(bind) => {
                line_checks("ui.bind", bind.span, owner, out);
            }
            DumpBinding::On(on) => {
                line_checks("ui.on", on.span, owner, out);
            }
            DumpBinding::Model(model) => {
                line_checks("ui.model", model.span, owner, out);
                for attribute in &model.attributes {
                    line_checks("attr", attribute.span, Some(("ui.model", model.span)), out);
                }
            }
            DumpBinding::SlotContent(content) => {
                line_checks("ui.slot-content", content.span, owner, out);
            }
            DumpBinding::VueDirective(directive) => {
                line_checks("vue.directive", directive.span, owner, out);
            }
            DumpBinding::VueCssBind(bind) => {
                line_checks("vue.css-bind", bind.span, owner, out);
            }
            DumpBinding::VueSync(sync) => {
                line_checks("vue.sync", sync.span, owner, out);
            }
            DumpBinding::VueSlotScope(scope) => {
                line_checks("vue.slot-scope", scope.span, owner, out);
            }
            DumpBinding::VueOnce(once) => {
                line_checks("vue.once", once.span, owner, out);
            }
            DumpBinding::VueMemo(memo) => {
                line_checks("vue.memo", memo.span, owner, out);
            }
            DumpBinding::VueShow(show) => {
                line_checks("vue.show", show.span, owner, out);
            }
            DumpBinding::VueHtml(html) => {
                line_checks("vue.html", html.span, owner, out);
            }
            DumpBinding::VueText(text) => {
                line_checks("vue.text", text.span, owner, out);
            }
            DumpBinding::VueCloak(cloak) => {
                line_checks("vue.cloak", cloak.span, owner, out);
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
fn visit_if(if_op: &DumpIf, rigor: Rigor, out: &mut Vec<Violation>) {
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
