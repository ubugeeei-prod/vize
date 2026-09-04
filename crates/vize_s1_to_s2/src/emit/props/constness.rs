//! What the codegen reads a bind value or handler *as*: whether it is a
//! runtime constant, and the text it prints. Split out of `props.rs` to
//! keep both files inside the source budget.

use vize_s0::String;

use super::super::js::js_expr_source;
use super::super::props_value::bind_value;
use super::static_expr::is_static_bound_expr;
use super::ts_view::ts_view;

/// `is_const_handler`: the handler text the codegen reads is a lone
/// constant binding name.
pub(in crate::emit) fn handler_is_constant(
    on: &vize_s2::op::OnOp<'_>,
    constant: &dyn Fn(&str) -> bool,
) -> bool {
    let Some(handler) = on.handler else {
        return false;
    };
    let source = match handler {
        vize_s2::expr::ExprRef::Js(js) => js.source,
        vize_s2::expr::ExprRef::Opaque(opaque) => opaque.source,
        vize_s2::expr::ExprRef::Foreign(_) | vize_s2::expr::ExprRef::Filter(_) => return false,
    };
    constant(source.trim())
}

/// `is_static_bound_expression` over the value the shipped codegen sees:
/// under `is_ts` the transform has already erased the types, so a
/// `{ … } as const` object is a *static* literal by the time the patch
/// flags and the hoist decisions read it.
pub(in crate::emit) fn bind_value_is_static_patchless(
    bind: &vize_s2::op::BindOp<'_>,
    is_ts: bool,
) -> bool {
    match bind_value(bind) {
        Ok(value) => value.js().is_some_and(|js| match ts_view(js, is_ts) {
            Some(view) => view.is_static(),
            None => is_static_bound_expr(js.ast),
        }),
        Err(_) => false,
    }
}

/// The bind value's text the way the shipped codegen writes it: the
/// authored source, or its type-erased spelling under `is_ts`.
pub(in crate::emit) fn bind_value_text(js: &vize_s2::expr::JsExpr<'_>, is_ts: bool) -> String {
    match ts_view(js, is_ts) {
        Some(view) => view.into_text(),
        None => String::from(js_expr_source(js).as_str()),
    }
}
