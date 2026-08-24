//! `vue.filter` → `_filter_*(...)` call text, re-admitted as JS.

use vize_carton::{Allocator, String};
use vize_disegno::expr::{ExprRef, VueFilterApp, VueFilterExpr};
use vize_disegno::op::{BindingOp, DynamicName, Op};

/// Rewrite every [`ExprRef::Filter`] in the tree into the Vue 2 wrap
/// (`a | f` → `_filter_f(a)`, `a | f(b)` → `_filter_f(a,b)`). Mixed
/// text-runs that absorbed a pipe into a compound opaque stay as
/// authored — those parts are not `ExprRef::Filter` (recorded gap).
pub(super) fn rewrite<'a>(allocator: &'a Allocator, ops: &mut [Op<'a>]) {
    for op in ops.iter_mut() {
        match op {
            Op::Element(element) => {
                rewrite_bindings(allocator, &mut element.bindings);
                rewrite(allocator, &mut element.children.ops);
            }
            Op::Component(component) => {
                rewrite_bindings(allocator, &mut component.bindings);
                rewrite(allocator, &mut component.children.ops);
            }
            Op::Slot(slot) => {
                rewrite_name(allocator, &mut slot.name);
                rewrite_bindings(allocator, &mut slot.bindings);
                rewrite(allocator, &mut slot.fallback.ops);
            }
            Op::Interpolation(interp) => rewrite_expr(allocator, &mut interp.expression),
            Op::If(if_op) => {
                for branch in if_op.branches.iter_mut() {
                    if let Some(condition) = &mut branch.condition {
                        rewrite_expr(allocator, condition);
                    }
                    rewrite(allocator, &mut branch.region.ops);
                }
            }
            Op::For(for_op) => {
                rewrite_expr(allocator, &mut for_op.binding.source);
                rewrite_expr(allocator, &mut for_op.binding.value);
                if let Some(key) = &mut for_op.binding.key {
                    rewrite_expr(allocator, key);
                }
                if let Some(index) = &mut for_op.binding.index {
                    rewrite_expr(allocator, index);
                }
                rewrite(allocator, &mut for_op.region.ops);
            }
            Op::Text(_) => {}
        }
    }
}

fn rewrite_bindings<'a>(allocator: &'a Allocator, bindings: &mut [BindingOp<'a>]) {
    for binding in bindings.iter_mut() {
        match binding {
            BindingOp::Bind(bind) => {
                if let Some(name) = &mut bind.name {
                    rewrite_name(allocator, name);
                }
                if let Some(value) = &mut bind.value {
                    rewrite_expr(allocator, value);
                }
            }
            BindingOp::On(on) => {
                if let Some(name) = &mut on.name {
                    rewrite_name(allocator, name);
                }
                if let Some(handler) = &mut on.handler {
                    rewrite_expr(allocator, handler);
                }
            }
            BindingOp::Model(model) => {
                rewrite_expr(allocator, &mut model.contract.read);
                rewrite_expr(allocator, &mut model.contract.write);
            }
            BindingOp::SlotContent(content) => {
                if let Some(name) = &mut content.name {
                    rewrite_name(allocator, name);
                }
                if let Some(params) = &mut content.params {
                    rewrite_expr(allocator, params);
                }
            }
            BindingOp::VueDirective(directive) => {
                if let Some(argument) = &mut directive.argument {
                    rewrite_name(allocator, argument);
                }
                if let Some(value) = &mut directive.value {
                    rewrite_expr(allocator, value);
                }
            }
            BindingOp::VueCssBind(bind) => rewrite_expr(allocator, &mut bind.value),
            BindingOp::VueSync(sync) => rewrite_expr(allocator, &mut sync.value),
            BindingOp::VueSlotScope(scope) => {
                if let Some(params) = &mut scope.params {
                    rewrite_expr(allocator, params);
                }
            }
            BindingOp::VueOnce(_) => {}
            BindingOp::VueMemo(memo) => rewrite_expr(allocator, &mut memo.value),
        }
    }
}

fn rewrite_name<'a>(allocator: &'a Allocator, name: &mut DynamicName<'a>) {
    if let DynamicName::Dynamic(expr) = name {
        rewrite_expr(allocator, expr);
    }
}

fn rewrite_expr<'a>(allocator: &'a Allocator, expr: &mut ExprRef<'a>) {
    let ExprRef::Filter(filter) = *expr else {
        return;
    };
    *expr = wrap(allocator, filter);
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
    let id = filter_id(app.name);
    match app.args {
        None => {
            let mut out = String::with_capacity(id.len() + exp.len() + 2);
            out.push_str(id.as_str());
            out.push('(');
            out.push_str(exp);
            out.push(')');
            out
        }
        Some("") => {
            let mut out = String::with_capacity(id.len() + exp.len() + 2);
            out.push_str(id.as_str());
            out.push('(');
            out.push_str(exp);
            out.push(')');
            out
        }
        Some(args) => {
            let mut out = String::with_capacity(id.len() + exp.len() + args.len() + 3);
            out.push_str(id.as_str());
            out.push('(');
            out.push_str(exp);
            out.push(',');
            out.push_str(args);
            out.push(')');
            out
        }
    }
}

/// `_filter_<name>` with `-` mapped to `_`, the shipped
/// `to_valid_asset_identifier("filter", name)` for identifier names.
fn filter_id(name: &str) -> String {
    let mut id = String::with_capacity(8 + name.len());
    id.push_str("_filter_");
    for c in name.chars() {
        if c == '-' {
            id.push('_');
        } else {
            id.push(c);
        }
    }
    id
}
