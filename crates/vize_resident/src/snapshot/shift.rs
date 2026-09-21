//! Moving an adopted region: every span of its ops and diagnostics shifts by
//! the distance the region moved inside its block. Matches are exhaustive
//! with no `_` arm on purpose — a new op, binding or expression variant must
//! break this file loudly, never be left unshifted.

use vize_davinci::diagnostic::Diagnostic;
use vize_s0::Span;
use vize_s2::folio::{
    FolioAttribute, FolioBinding, FolioExpr, FolioForBinding, FolioName, FolioOp,
};

use super::region::RegionLowering;

/// A copy of `lowering` whose spans moved from `from` to `to`.
#[must_use]
pub fn shifted(lowering: &RegionLowering, from: u32, to: u32) -> RegionLowering {
    let mut moved = lowering.clone();
    let delta = i64::from(to) - i64::from(from);
    let mut shift = |span: &mut Span| {
        span.start = offset(span.start, delta);
        span.end = offset(span.end, delta);
    };
    for op in &mut moved.ops {
        shift_op(op, &mut shift);
    }
    for diagnostic in moved.surface.iter_mut().chain(&mut moved.semantic) {
        shift_diagnostic(diagnostic, &mut shift);
    }
    moved
}

fn offset(position: u32, delta: i64) -> u32 {
    u32::try_from(i64::from(position) + delta).expect("a moved span stays inside its block")
}

type Shift<'s> = dyn FnMut(&mut Span) + 's;

fn shift_diagnostic(diagnostic: &mut Diagnostic, shift: &mut Shift<'_>) {
    shift(&mut diagnostic.span);
    for part in &mut diagnostic.parts {
        shift(&mut part.span);
    }
}

fn shift_op(op: &mut FolioOp, shift: &mut Shift<'_>) {
    match op {
        FolioOp::Element(element) => {
            shift(&mut element.span);
            shift_owner(&mut element.attributes, &mut element.bindings, shift);
            shift_ops(&mut element.children, shift);
        }
        FolioOp::Component(component) => {
            shift(&mut component.span);
            shift_owner(&mut component.attributes, &mut component.bindings, shift);
            shift_ops(&mut component.children, shift);
        }
        FolioOp::Text(text) => shift(&mut text.span),
        FolioOp::Interpolation(interpolation) => {
            shift(&mut interpolation.span);
            shift_expr(&mut interpolation.expression, shift);
        }
        FolioOp::Comment(comment) => shift(&mut comment.span),
        FolioOp::If(if_op) => {
            shift(&mut if_op.span);
            for branch in &mut if_op.branches {
                shift(&mut branch.span);
                if let Some(condition) = &mut branch.condition {
                    shift_expr(condition, shift);
                }
                shift_ops(&mut branch.ops, shift);
            }
        }
        FolioOp::For(for_op) => {
            shift(&mut for_op.span);
            shift_for_binding(&mut for_op.binding, shift);
            shift_ops(&mut for_op.ops, shift);
        }
        FolioOp::Slot(slot) => {
            shift(&mut slot.span);
            shift_name(&mut slot.name, shift);
            shift_owner(&mut slot.attributes, &mut slot.bindings, shift);
            shift_ops(&mut slot.fallback, shift);
        }
    }
}

fn shift_ops(ops: &mut [FolioOp], shift: &mut Shift<'_>) {
    for op in ops {
        shift_op(op, shift);
    }
}

fn shift_owner(
    attributes: &mut [FolioAttribute],
    bindings: &mut [FolioBinding],
    shift: &mut Shift<'_>,
) {
    shift_attributes(attributes, shift);
    for binding in bindings {
        shift_binding(binding, shift);
    }
}

fn shift_attributes(attributes: &mut [FolioAttribute], shift: &mut Shift<'_>) {
    for attribute in attributes {
        shift(&mut attribute.span);
    }
}

fn shift_binding(binding: &mut FolioBinding, shift: &mut Shift<'_>) {
    match binding {
        FolioBinding::Bind(bind) => {
            shift(&mut bind.span);
            shift_opt_name(&mut bind.name, shift);
            shift_opt_expr(&mut bind.value, shift);
        }
        FolioBinding::On(on) => {
            shift(&mut on.span);
            shift_opt_name(&mut on.name, shift);
            shift_opt_expr(&mut on.handler, shift);
        }
        FolioBinding::Model(model) => {
            shift(&mut model.span);
            shift_expr(&mut model.contract.read, shift);
            shift_expr(&mut model.contract.write, shift);
            shift_opt_name(&mut model.argument, shift);
            shift_attributes(&mut model.attributes, shift);
        }
        FolioBinding::SlotContent(content) => {
            shift(&mut content.span);
            shift_opt_name(&mut content.name, shift);
            shift_opt_expr(&mut content.params, shift);
        }
        FolioBinding::VueDirective(directive) => {
            shift(&mut directive.span);
            shift_opt_name(&mut directive.argument, shift);
            shift_opt_expr(&mut directive.value, shift);
        }
        FolioBinding::VueCssBind(bind) => {
            shift(&mut bind.span);
            shift_expr(&mut bind.value, shift);
        }
        FolioBinding::VueSync(sync) => {
            shift(&mut sync.span);
            shift_expr(&mut sync.value, shift);
        }
        FolioBinding::VueSlotScope(scope) => {
            shift(&mut scope.span);
            shift_opt_expr(&mut scope.params, shift);
        }
        FolioBinding::VueOnce(once) => shift(&mut once.span),
        FolioBinding::VueMemo(memo) => {
            shift(&mut memo.span);
            shift_expr(&mut memo.value, shift);
        }
        FolioBinding::VueShow(show) => {
            shift(&mut show.span);
            shift_expr(&mut show.value, shift);
        }
        FolioBinding::VueHtml(html) => {
            shift(&mut html.span);
            shift_opt_expr(&mut html.value, shift);
        }
        FolioBinding::VueText(text) => {
            shift(&mut text.span);
            shift_opt_expr(&mut text.value, shift);
        }
        FolioBinding::VueCloak(cloak) => shift(&mut cloak.span),
    }
}

fn shift_for_binding(binding: &mut FolioForBinding, shift: &mut Shift<'_>) {
    shift_expr(&mut binding.source, shift);
    shift_expr(&mut binding.value, shift);
    shift_opt_expr(&mut binding.key, shift);
    shift_opt_expr(&mut binding.index, shift);
}

fn shift_name(name: &mut FolioName, shift: &mut Shift<'_>) {
    match name {
        FolioName::Static(_) => {}
        FolioName::Dynamic(expr) => shift_expr(expr, shift),
    }
}

fn shift_opt_name(name: &mut Option<FolioName>, shift: &mut Shift<'_>) {
    if let Some(name) = name {
        shift_name(name, shift);
    }
}

fn shift_opt_expr(expr: &mut Option<FolioExpr>, shift: &mut Shift<'_>) {
    if let Some(expr) = expr {
        shift_expr(expr, shift);
    }
}

fn shift_expr(expr: &mut FolioExpr, shift: &mut Shift<'_>) {
    match expr {
        FolioExpr::Js { span, .. }
        | FolioExpr::Opaque { span, .. }
        | FolioExpr::Foreign { span, .. }
        | FolioExpr::Filter { span, .. } => shift(span),
    }
}
