//! `vue.filter` → `_filter_*(...)` call text, re-admitted as JS.

use alloc::vec::Vec as StdVec;

use vize_s0::{Allocator, String};
use vize_s2::expr::{ExprRef, VueFilterApp, VueFilterExpr};
use vize_s2::op::{BindingOp, DynamicName, Op};

use crate::emit::js::asset_ident;

/// Rewrite every [`ExprRef::Filter`] in the tree into the Vue 2 wrap
/// (`a | f` → `_filter_f(a)`, `a | f(b)` → `_filter_f(a,b)`). Mixed
/// text-runs that absorbed a pipe into a compound opaque stay as
/// authored — those parts are not `ExprRef::Filter` (recorded gap).
pub(super) fn rewrite<'a>(
    allocator: &'a Allocator,
    ops: &mut [Op<'a>],
    filters: &mut StdVec<String>,
) {
    for op in ops.iter_mut() {
        match op {
            Op::Element(element) => {
                rewrite_bindings(allocator, &mut element.bindings, filters);
                rewrite(allocator, &mut element.children.ops, filters);
            }
            Op::Component(component) => {
                rewrite_bindings(allocator, &mut component.bindings, filters);
                rewrite(allocator, &mut component.children.ops, filters);
            }
            Op::Slot(slot) => {
                rewrite_name(allocator, &mut slot.name, filters);
                rewrite_bindings(allocator, &mut slot.bindings, filters);
                rewrite(allocator, &mut slot.fallback.ops, filters);
            }
            Op::Interpolation(interp) => rewrite_expr(allocator, &mut interp.expression, filters),
            Op::If(if_op) => {
                for branch in if_op.branches.iter_mut() {
                    if let Some(condition) = &mut branch.condition {
                        rewrite_expr(allocator, condition, filters);
                    }
                    rewrite(allocator, &mut branch.region.ops, filters);
                }
            }
            Op::For(for_op) => {
                rewrite_expr(allocator, &mut for_op.binding.source, filters);
                rewrite_expr(allocator, &mut for_op.binding.value, filters);
                if let Some(key) = &mut for_op.binding.key {
                    rewrite_expr(allocator, key, filters);
                }
                if let Some(index) = &mut for_op.binding.index {
                    rewrite_expr(allocator, index, filters);
                }
                rewrite(allocator, &mut for_op.region.ops, filters);
            }
            Op::Text(_) => {}
        }
    }
}

fn rewrite_bindings<'a>(
    allocator: &'a Allocator,
    bindings: &mut [BindingOp<'a>],
    filters: &mut StdVec<String>,
) {
    for binding in bindings.iter_mut() {
        match binding {
            BindingOp::Bind(bind) => {
                if let Some(name) = &mut bind.name {
                    rewrite_name(allocator, name, filters);
                }
                if let Some(value) = &mut bind.value {
                    rewrite_expr(allocator, value, filters);
                }
            }
            BindingOp::On(on) => {
                if let Some(name) = &mut on.name {
                    rewrite_name(allocator, name, filters);
                }
                if let Some(handler) = &mut on.handler {
                    rewrite_expr(allocator, handler, filters);
                }
            }
            BindingOp::Model(model) => {
                rewrite_expr(allocator, &mut model.contract.read, filters);
                rewrite_expr(allocator, &mut model.contract.write, filters);
            }
            BindingOp::SlotContent(content) => {
                if let Some(name) = &mut content.name {
                    rewrite_name(allocator, name, filters);
                }
                if let Some(params) = &mut content.params {
                    rewrite_expr(allocator, params, filters);
                }
            }
            BindingOp::VueDirective(directive) => {
                if let Some(argument) = &mut directive.argument {
                    rewrite_name(allocator, argument, filters);
                }
                if let Some(value) = &mut directive.value {
                    rewrite_expr(allocator, value, filters);
                }
            }
            BindingOp::VueCssBind(bind) => rewrite_expr(allocator, &mut bind.value, filters),
            BindingOp::VueSync(sync) => rewrite_expr(allocator, &mut sync.value, filters),
            BindingOp::VueSlotScope(scope) => {
                if let Some(params) = &mut scope.params {
                    rewrite_expr(allocator, params, filters);
                }
            }
            BindingOp::VueOnce(_) => {}
            BindingOp::VueMemo(memo) => rewrite_expr(allocator, &mut memo.value, filters),
        }
    }
}

fn rewrite_name<'a>(
    allocator: &'a Allocator,
    name: &mut DynamicName<'a>,
    filters: &mut StdVec<String>,
) {
    if let DynamicName::Dynamic(expr) = name {
        rewrite_expr(allocator, expr, filters);
    }
}

fn rewrite_expr<'a>(
    allocator: &'a Allocator,
    expr: &mut ExprRef<'a>,
    filters: &mut StdVec<String>,
) {
    let ExprRef::Filter(filter) = *expr else {
        return;
    };
    record_filters(filter, filters);
    *expr = wrap(allocator, filter);
}

fn record_filters(filter: &VueFilterExpr<'_>, filters: &mut StdVec<String>) {
    for app in &filter.filters {
        if !filters.iter().any(|seen| seen.as_str() == app.name) {
            filters.push(String::from(app.name));
        }
    }
}

fn wrap<'a>(allocator: &'a Allocator, filter: &VueFilterExpr<'a>) -> ExprRef<'a> {
    let mut out = String::from(filter.base.source());
    for app in &filter.filters {
        out = wrap_one(out.as_str(), app);
    }
    let text = allocator.alloc_str(out.as_str());
    ExprRef::parse_js_in(allocator, text, filter.span)
}

fn wrap_one(exp: &str, app: &VueFilterApp<'_>) -> String {
    let id = asset_ident("filter", app.name);
    match app.raw.find('(') {
        None => {
            let mut out = String::with_capacity(id.len() + exp.len() + 2);
            out.push_str(id.as_str());
            out.push('(');
            out.push_str(exp);
            out.push(')');
            out
        }
        Some(idx) if &app.raw[idx + 1..] == ")" => {
            let mut out = String::with_capacity(id.len() + exp.len() + 2);
            out.push_str(id.as_str());
            out.push('(');
            out.push_str(exp);
            out.push(')');
            out
        }
        Some(idx) => {
            let args = &app.raw[idx + 1..];
            let mut out = String::with_capacity(id.len() + exp.len() + args.len() + 3);
            out.push_str(id.as_str());
            out.push('(');
            out.push_str(exp);
            out.push(',');
            out.push_str(args);
            out
        }
    }
}
