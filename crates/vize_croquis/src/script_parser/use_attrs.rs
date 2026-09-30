//! Recognize Vue's `useAttrs` helper after all imports and bindings are known.

use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk};

use super::ScriptParseResult;

pub(super) fn has_vue_use_attrs_call(
    result: &ScriptParseResult,
    program: &Program<'_>,
    source: &str,
) -> bool {
    // Both an aliased import and a namespace member contain this export name.
    // Avoid a second AST walk for the usual case without useAttrs.
    if !source.contains("useAttrs") {
        return false;
    }
    let mut visitor = AttrsCallVisitor {
        result,
        found: false,
    };
    visitor.visit_program(program);
    visitor.found
}

struct AttrsCallVisitor<'r> {
    result: &'r ScriptParseResult,
    found: bool,
}

impl<'a> Visit<'a> for AttrsCallVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if !self.found {
            self.found = is_vue_use_attrs_call(self.result, call);
        }
        if !self.found {
            walk::walk_call_expression(self, call);
        }
    }
}

fn is_vue_use_attrs_call(result: &ScriptParseResult, call: &CallExpression<'_>) -> bool {
    match &call.callee {
        Expression::Identifier(id) => {
            let local = id.name.as_str();
            imported_vue_export(result, local, "useAttrs", call.span.start)
                || (local == "useAttrs"
                    && !result.bindings.bindings.contains_key(local)
                    && !has_local_shadow(result, local, call.span.start))
        }
        Expression::StaticMemberExpression(member) if member.property.name == "useAttrs" => {
            let Expression::Identifier(object) = &member.object else {
                return false;
            };
            imported_vue_export(result, object.name.as_str(), "*", call.span.start)
        }
        _ => false,
    }
}

fn imported_vue_export(result: &ScriptParseResult, local: &str, export: &str, offset: u32) -> bool {
    result
        .import_sources
        .get(local)
        .is_some_and(|source| source.as_str() == "vue")
        && result.types.definitions().imported_type_export(local) == Some(export)
        && !has_local_shadow(result, local, offset)
}

fn has_local_shadow(result: &ScriptParseResult, local: &str, offset: u32) -> bool {
    result
        .scopes
        .bindings_visible_at(offset)
        .into_iter()
        .find(|(name, _, _)| *name == local)
        .is_some_and(|(_, binding, _)| {
            !result.import_statements.iter().any(|import| {
                import.start <= binding.declaration_offset
                    && binding.declaration_offset <= import.end
            })
        })
}
