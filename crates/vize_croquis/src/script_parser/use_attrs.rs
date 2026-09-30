//! Recognize Vue's `useAttrs` helper after all imports and bindings are known.

use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;

use super::ScriptParseResult;

pub(super) fn has_vue_use_attrs_call(
    result: &ScriptParseResult,
    program: &Program<'_>,
    source: &str,
) -> bool {
    // Imports are known already. The byte probe also catches unimported calls
    // and escaped identifiers without visiting the AST for unrelated scripts.
    if !result.uses_attrs_call && !has_source_use_attrs_candidate(source) {
        return false;
    }
    let mut candidate_names = vec!["useAttrs"];
    for (local, import_source) in &result.import_sources {
        if import_source.as_str() == "vue"
            && matches!(
                result
                    .types
                    .definitions()
                    .imported_type_export(local.as_str()),
                Some("useAttrs" | "*")
            )
        {
            candidate_names.push(local.as_str());
        }
    }

    let mut visitor = AttrsCallVisitor {
        result,
        found: false,
    };
    // The parser already walked the whole program for bindings. Search only
    // statements that mention a possible callee instead of walking every AST
    // node a second time (the large-script instruction budget is tight).
    for statement in &program.body {
        let span = statement.span();
        if source
            .get(span.start as usize..span.end as usize)
            .is_some_and(|text| {
                text.contains('\\') || candidate_names.iter().any(|name| text.contains(name))
            })
        {
            visitor.visit_statement(statement);
            if visitor.found {
                break;
            }
        }
    }
    visitor.found
}

fn has_source_use_attrs_candidate(source: &str) -> bool {
    let bytes = source.as_bytes();
    if memchr::memchr(b'\\', bytes).is_some() {
        return true;
    }
    memchr::memchr_iter(b'A', bytes)
        .any(|pos| pos >= 3 && bytes.get(pos - 3..pos + 5) == Some(b"useAttrs".as_slice()))
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
