use vize_s2::expr::ExprRef;
use vize_s2::op::OnOp;

use super::super::{EmitCx, options::BindingKind};

/// `needs_von_handler_cache`: the option is on, no template-scope param
/// is in play, the directive has an expression, and that expression is
/// not a bare `SetupConst` reference.
pub(in crate::emit) fn needs_handler_cache(cx: &EmitCx<'_>, on: &OnOp<'_>) -> bool {
    let Some(handler) = on.handler else {
        return false;
    };
    if !cx.caches_handlers() {
        return false;
    }
    !is_setup_const_handler(cx, &handler)
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
