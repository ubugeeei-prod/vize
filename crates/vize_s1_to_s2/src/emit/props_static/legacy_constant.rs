use oxc_ast::ast::{self as js, Expression, IdentifierReference};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{Allocator, String};
use vize_s2::expr::JsExpr;

use super::super::props::ts_view;
use super::super::props_object::shipped_constant;

/// The shipped transform strips a typed callback parameter before the props
/// hoist check, but leaves a typed local declaration in a callback body
/// untouched. Inspect the retained declaration binding so the S2 emitter
/// does not hoist a props object the shipped transform leaves inline.
fn has_typed_local_declaration(expr: &Expression<'_>) -> bool {
    let mut walk = TypedLocalDeclaration { found: false };
    walk.visit_expression(expr);
    walk.found
}

struct TypedLocalDeclaration {
    found: bool,
}

impl<'a> Visit<'a> for TypedLocalDeclaration {
    fn visit_variable_declarator(&mut self, declarator: &js::VariableDeclarator<'a>) {
        self.found |= declarator.type_annotation.is_some();
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
    }
}

/// Match the shipped props-hoist constant test on the expression bytes it
/// reads. A typed parameter is erased first, then the normal local/global
/// dependency walk decides whether the callback is constant.
pub(super) fn shipped_component_hoist_constant(js: &JsExpr<'_>, is_ts: bool) -> bool {
    // The shipped JS lane cannot classify TypeScript-only syntax as a
    // constant. With `is_ts`, its transform may erase those types first.
    if !is_ts
        && (super::super::on_typed::uses_ts_only_syntax(js.ast)
            || has_typed_local_declaration(js.ast))
    {
        return false;
    }
    let Some(view) = ts_view(js, is_ts) else {
        // A typed declaration that survives the TS strip remains dynamic.
        if is_ts && has_typed_local_declaration(js.ast) {
            return false;
        }
        return legacy_global_constant_expr(js.ast, js.source)
            || self_bound_constant_expr(js.ast, js.source);
    };
    let source = view.into_text();
    let mut wrapped = String::with_capacity(source.len() + 2);
    wrapped.push('(');
    wrapped.push_str(source.as_str());
    wrapped.push(')');
    let allocator = Allocator::new();
    Parser::new(
        allocator.as_oxc(),
        wrapped.as_str(),
        SourceType::default().with_module(true),
    )
    .parse_expression()
    .ok()
    .and_then(|expr| shipped_constant(&expr, source.as_str()))
    .is_some()
}

pub(super) fn legacy_global_constant_expr(expr: &Expression<'_>, source: &str) -> bool {
    if source.contains("_ctx.")
        || source.contains("$setup.")
        || source.contains("__props.")
        || source.contains("$props.")
    {
        return false;
    }
    let mut walk = LegacyGlobalConstWalk { dynamic: false };
    walk.visit_expression(expr);
    !walk.dynamic
}

struct LegacyGlobalConstWalk {
    dynamic: bool,
}

impl<'a> Visit<'a> for LegacyGlobalConstWalk {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if !super::super::props_bind::is_global_key_name(ident.name.as_str()) {
            self.dynamic = true;
        }
    }
}

/// The self-bound half of the shipped constant rule. Free names stay
/// dynamic; see [`crate::pass::hoist::constant_for_hoist`].
pub(super) fn self_bound_constant_expr(expr: &Expression<'_>, source: &str) -> bool {
    crate::pass::hoist::self_bound_js_constant(expr, source)
}
