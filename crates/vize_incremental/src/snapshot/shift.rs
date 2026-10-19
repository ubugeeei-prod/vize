//! Moving an adopted region: every span of its ops and diagnostics shifts by
//! the distance the region moved inside its block. Matches are exhaustive
//! with no `_` arm on purpose — a new op, binding or expression variant must
//! break this file loudly, never be left unshifted.

use vize_l0::Span;
use vize_l0::diag::Diagnostic;
use vize_l2::dump::{
    Attribute as DumpAttribute, Binding as DumpBinding, Expr as DumpExpr,
    ForBinding as DumpForBinding, Name as DumpName, Op as DumpOp,
};

use super::region::RegionLowering;

/// A copy of `lowering` whose spans moved from `from` to `to`, or `None` when
/// a moved span would leave the `u32` offset range.
#[must_use]
pub fn shifted(lowering: &RegionLowering, from: u32, to: u32) -> Option<RegionLowering> {
    let mut moved = lowering.clone();
    let delta = i64::from(to) - i64::from(from);
    let mut in_range = true;
    let mut shift = |span: &mut Span| {
        for position in [&mut span.start, &mut span.end] {
            match u32::try_from(i64::from(*position) + delta) {
                Ok(moved) => *position = moved,
                Err(_) => in_range = false,
            }
        }
    };
    for op in &mut moved.ops {
        shift_op(op, &mut shift);
    }
    for diagnostic in moved.surface.iter_mut().chain(&mut moved.semantic) {
        shift_diagnostic(diagnostic, &mut shift);
    }
    in_range.then_some(moved)
}

type Shift<'s> = dyn FnMut(&mut Span) + 's;

fn shift_diagnostic(diagnostic: &mut Diagnostic, shift: &mut Shift<'_>) {
    shift(&mut diagnostic.span);
    for part in &mut diagnostic.parts {
        shift(&mut part.span);
    }
}

fn shift_op(op: &mut DumpOp, shift: &mut Shift<'_>) {
    match op {
        DumpOp::Element(element) => {
            shift(&mut element.span);
            shift_owner(&mut element.attributes, &mut element.bindings, shift);
            shift_ops(&mut element.children, shift);
        }
        DumpOp::Component(component) => {
            shift(&mut component.span);
            shift_owner(&mut component.attributes, &mut component.bindings, shift);
            shift_ops(&mut component.children, shift);
        }
        DumpOp::Text(text) => shift(&mut text.span),
        DumpOp::Interpolation(interpolation) => {
            shift(&mut interpolation.span);
            shift_expr(&mut interpolation.expression, shift);
        }
        DumpOp::Comment(comment) => shift(&mut comment.span),
        DumpOp::If(if_op) => {
            shift(&mut if_op.span);
            for branch in &mut if_op.branches {
                shift(&mut branch.span);
                if let Some(condition) = &mut branch.condition {
                    shift_expr(condition, shift);
                }
                shift_ops(&mut branch.ops, shift);
            }
        }
        DumpOp::For(for_op) => {
            shift(&mut for_op.span);
            shift_for_binding(&mut for_op.binding, shift);
            shift_ops(&mut for_op.ops, shift);
        }
        DumpOp::OriginalFor(for_op) => {
            shift(&mut for_op.span);
            shift_ops(&mut for_op.ops, shift);
        }
        DumpOp::Slot(slot) => {
            shift(&mut slot.span);
            shift_name(&mut slot.name, shift);
            shift_owner(&mut slot.attributes, &mut slot.bindings, shift);
            shift_ops(&mut slot.fallback, shift);
        }
    }
}

fn shift_ops(ops: &mut [DumpOp], shift: &mut Shift<'_>) {
    for op in ops {
        shift_op(op, shift);
    }
}

fn shift_owner(
    attributes: &mut [DumpAttribute],
    bindings: &mut [DumpBinding],
    shift: &mut Shift<'_>,
) {
    shift_attributes(attributes, shift);
    for binding in bindings {
        shift_binding(binding, shift);
    }
}

fn shift_attributes(attributes: &mut [DumpAttribute], shift: &mut Shift<'_>) {
    for attribute in attributes {
        shift(&mut attribute.span);
    }
}

fn shift_binding(binding: &mut DumpBinding, shift: &mut Shift<'_>) {
    match binding {
        DumpBinding::Bind(bind) => {
            shift(&mut bind.span);
            shift_opt_name(&mut bind.name, shift);
            shift_opt_expr(&mut bind.value, shift);
        }
        DumpBinding::On(on) => {
            shift(&mut on.span);
            shift_opt_name(&mut on.name, shift);
            shift_opt_expr(&mut on.handler, shift);
        }
        DumpBinding::Model(model) => {
            shift(&mut model.span);
            shift_expr(&mut model.contract.read, shift);
            shift_expr(&mut model.contract.write, shift);
            shift_opt_name(&mut model.argument, shift);
            shift_attributes(&mut model.attributes, shift);
        }
        DumpBinding::SlotContent(content) => {
            shift(&mut content.span);
            shift_opt_name(&mut content.name, shift);
            shift_opt_expr(&mut content.params, shift);
        }
        DumpBinding::VueDirective(directive) => {
            shift(&mut directive.span);
            shift_opt_name(&mut directive.argument, shift);
            shift_opt_expr(&mut directive.value, shift);
        }
        DumpBinding::SlotsObject(object) => {
            shift(&mut object.span);
            shift_expr(&mut object.value, shift);
        }
        DumpBinding::VueCssBind(bind) => {
            shift(&mut bind.span);
            shift_expr(&mut bind.value, shift);
        }
        DumpBinding::VueSync(sync) => {
            shift(&mut sync.span);
            shift_expr(&mut sync.value, shift);
        }
        DumpBinding::VueSlotScope(scope) => {
            shift(&mut scope.span);
            shift_opt_expr(&mut scope.params, shift);
        }
        DumpBinding::VueOnce(once) => shift(&mut once.span),
        DumpBinding::VueMemo(memo) => {
            shift(&mut memo.span);
            shift_expr(&mut memo.value, shift);
        }
        DumpBinding::VueShow(show) => {
            shift(&mut show.span);
            shift_expr(&mut show.value, shift);
        }
        DumpBinding::VueHtml(html) => {
            shift(&mut html.span);
            shift_opt_expr(&mut html.value, shift);
        }
        DumpBinding::VueText(text) => {
            shift(&mut text.span);
            shift_opt_expr(&mut text.value, shift);
        }
        DumpBinding::VueCloak(cloak) => shift(&mut cloak.span),
    }
}

fn shift_for_binding(binding: &mut DumpForBinding, shift: &mut Shift<'_>) {
    shift_expr(&mut binding.source, shift);
    shift_expr(&mut binding.value, shift);
    shift_opt_expr(&mut binding.key, shift);
    shift_opt_expr(&mut binding.index, shift);
}

fn shift_name(name: &mut DumpName, shift: &mut Shift<'_>) {
    match name {
        DumpName::Static(_) => {}
        DumpName::Dynamic(expr) => shift_expr(expr, shift),
    }
}

fn shift_opt_name(name: &mut Option<DumpName>, shift: &mut Shift<'_>) {
    if let Some(name) = name {
        shift_name(name, shift);
    }
}

fn shift_opt_expr(expr: &mut Option<DumpExpr>, shift: &mut Shift<'_>) {
    if let Some(expr) = expr {
        shift_expr(expr, shift);
    }
}

fn shift_expr(expr: &mut DumpExpr, shift: &mut Shift<'_>) {
    match expr {
        DumpExpr::Js { span, .. }
        | DumpExpr::Opaque { span, .. }
        | DumpExpr::Foreign { span, .. }
        | DumpExpr::Filter { span, .. } => shift(span),
    }
}

#[cfg(test)]
mod tests {
    //! The diagnostic original-For mirror moves its region without a head copy.

    use super::shifted;
    use crate::snapshot::region::RegionLowering;
    use vize_l0::{Span, String};
    use vize_l2::dump::{Expr, Interpolation, Op, OriginalFor};

    fn original_region(start: u32) -> RegionLowering {
        RegionLowering {
            ops: vec![Op::OriginalFor(OriginalFor {
                node: 7,
                span: Span::new(start, start + 12),
                ops: vec![Op::OriginalFor(OriginalFor {
                    node: 8,
                    span: Span::new(start + 3, start + 11),
                    ops: vec![Op::Interpolation(Interpolation {
                        span: Span::new(start + 6, start + 9),
                        expression: Expr::Js {
                            source: String::from("x"),
                            span: Span::new(start + 7, start + 8),
                        },
                    })],
                })],
            })],
            surface: Vec::new(),
            semantic: Vec::new(),
        }
    }

    #[test]
    fn original_regions_shift_once_without_changing_diagnostic_node_numbers() {
        let original = original_region(10);
        assert_eq!(shifted(&original, 10, 20), Some(original_region(20)));
        assert_eq!(shifted(&original, 10, 0), Some(original_region(0)));
        assert_eq!(original, original_region(10));
    }

    #[test]
    fn out_of_range_original_region_spans_refuse_adoption() {
        let original = original_region(10);
        assert_eq!(shifted(&original, 10, u32::MAX - 2), None);
        assert_eq!(shifted(&original, 11, 0), None);
    }
}
