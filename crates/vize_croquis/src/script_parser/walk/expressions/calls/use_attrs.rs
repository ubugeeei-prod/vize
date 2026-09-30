//! Recognize Vue's `useAttrs` helper from a call's imported binding.

use super::{CallExpression, Expression, ScriptParseResult};

pub(super) fn is_vue_use_attrs_call(result: &ScriptParseResult, call: &CallExpression<'_>) -> bool {
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
