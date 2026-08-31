use oxc_ast::ast::{Expression, IdentifierReference};
use oxc_ast_visit::Visit;

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
