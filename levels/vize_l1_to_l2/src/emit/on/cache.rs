use vize_l2::expr::ExprRef;
use vize_l2::op::OnOp;

use super::super::{EmitCx, options::BindingKind};

/// `needs_von_handler_cache`: the option is on, no template-scope param
/// is in play, the directive has an expression, and that expression is
/// not a bare `SetupConst` reference.
pub(in crate::emit) fn needs_handler_cache(cx: &EmitCx<'_>, on: &OnOp<'_>) -> bool {
    caches_owned_handler(cx, on, cx.component_props)
}

/// [`needs_handler_cache`] for an owner known to be a component or not.
pub(in crate::emit) fn caches_owned_handler(
    cx: &EmitCx<'_>,
    on: &OnOp<'_>,
    is_component: bool,
) -> bool {
    let Some(handler) = on.expression() else {
        return false;
    };
    if !cx.caches_handlers() {
        return false;
    }
    !is_setup_const_handler(cx, &handler)
        && !(is_component && bypasses_component_cache(cx, &handler))
}

/// A component receives a method reference as the authored function, so a
/// cached wrapper cannot hide its arity. Options API refs keep their
/// live-lookup wrapper.
pub(in crate::emit) fn bypasses_component_cache(cx: &EmitCx<'_>, expr: &ExprRef<'_>) -> bool {
    super::wrapped::options_api_handler_name(cx, expr).is_none()
        && super::super::prefix::handler_is_reference(cx.source, expr)
}

/// `is_setup_const_handler`: the processed handler text is a simple
/// identifier the binding table records as `SetupConst`. Only an inlined
/// render function leaves the name bare enough to read.
fn is_setup_const_handler(cx: &EmitCx<'_>, expr: &ExprRef<'_>) -> bool {
    let source = match expr {
        ExprRef::Js(js) => js.source,
        ExprRef::Opaque(opaque) => opaque.source,
        ExprRef::Foreign(_) | ExprRef::Filter(_) => return false,
    }
    .trim();
    if !cx.scope.inline() || !super::super::prefix::is_simple_identifier(source) {
        return false;
    }
    cx.scope
        .bindings()
        .and_then(|table| table.kind(source))
        .is_some_and(|kind| kind == BindingKind::SetupConst)
}
