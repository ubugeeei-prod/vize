//! Recognize Vue's `useAttrs` helper after all imports and bindings are known.

use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, SourceType};

use super::{
    ScriptParseResult, ScriptParserOptions, parse_script_setup_with_generic_and_jsx,
    parse_script_with_options_and_jsx,
};

/// Check for Vue's `useAttrs` only when a consumer needs the answer. The
/// ordinary Croquis script walk stays independent of this cross-file lint fact.
pub fn source_uses_vue_attrs(source: &str, setup: bool, jsx: bool) -> bool {
    if !source.contains("useAttrs") && !source.contains('\\') {
        return false;
    }

    let result = if setup {
        parse_script_setup_with_generic_and_jsx(source, None, jsx)
    } else {
        parse_script_with_options_and_jsx(source, ScriptParserOptions::default(), jsx)
    };
    let allocator = Allocator::default();
    let path = if jsx { "script.tsx" } else { "script.ts" };
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let parsed = super::recovery::parse_program_for_analysis(&allocator, source, source_type);
    !parsed.panicked && has_vue_use_attrs_call(&result, &parsed.program, source)
}

pub(super) fn has_vue_use_attrs_call(
    result: &ScriptParseResult,
    program: &Program<'_>,
    source: &str,
) -> bool {
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
