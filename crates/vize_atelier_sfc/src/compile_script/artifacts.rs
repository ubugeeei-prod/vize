//! Compile-time macro artifact extraction.
//!
//! These helpers keep ecosystem macro output independent from any specific
//! bundler hook. The SFC compiler can erase the runtime call while still
//! returning a loadable artifact for tools such as file-based routers.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, ToCompactString};
use vize_croquis::macros::{artifact_macro_names, macro_artifact_kind};

use crate::module_map::{Runs, apply_edits};
use crate::types::SfcMacroArtifact;

use self::imports::{
    artifact_macro_import_removal_spans, collect_artifact_macro_import_bindings,
    collect_static_imports, is_artifact_macro_only_import,
};
use super::runtime_bindings::collect_runtime_bindings;

mod imports;

pub(crate) fn extract_macro_artifacts(
    content: &str,
    absolute_offset: usize,
    nuxt_page_meta: bool,
) -> Vec<SfcMacroArtifact> {
    if !contains_artifact_macro_candidate(content) {
        return Vec::new();
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();

    if ret.panicked {
        return Vec::new();
    }

    let static_imports = collect_static_imports(ret.program.body.iter(), content, nuxt_page_meta);
    let mut runtime_bindings = collect_runtime_bindings(ret.program.body.iter());
    let macro_imports =
        collect_artifact_macro_import_bindings(ret.program.body.iter(), nuxt_page_meta);
    for name in &macro_imports {
        runtime_bindings.remove(name);
    }
    let mut artifacts = Vec::new();

    for stmt in ret.program.body.iter() {
        let Some(call) = artifact_call_from_statement(stmt) else {
            continue;
        };
        let Some(name) = call_name(call) else {
            continue;
        };
        if runtime_bindings.contains(name)
            || (name == "definePageMeta" && !nuxt_page_meta && !macro_imports.contains(name))
        {
            continue;
        }
        let Some(kind) = macro_artifact_kind(name) else {
            continue;
        };

        let start = call.span.start as usize;
        let end = call.span.end as usize;
        let Some(source) = content.get(start..end) else {
            continue;
        };
        let source = source.to_compact_string();
        let payload = call
            .arguments
            .first()
            .map(|arg| argument_source(arg, content))
            .filter(|source| !source.trim().is_empty())
            .unwrap_or_else(|| "{}".into());
        let module_code = build_artifact_module(kind, &payload, &static_imports);

        artifacts.push(SfcMacroArtifact {
            kind: kind.into(),
            name: name.into(),
            source,
            content: payload,
            module_code: Some(module_code),
            start: absolute_offset + start,
            end: absolute_offset + end,
        });
    }

    artifacts
}

pub(crate) fn erase_artifact_macro_statements(content: &str) -> Option<String> {
    erase_artifact_macro_statements_traced(content, false).map(|(erased, _)| erased)
}

/// [`erase_artifact_macro_statements`] with the erased text's provenance in
/// `content` (Davinci P3-9 source maps).
pub(crate) fn erase_artifact_macro_statements_traced(
    content: &str,
    nuxt_page_meta: bool,
) -> Option<(String, Runs)> {
    if !contains_artifact_macro_candidate(content) {
        return None;
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();

    if ret.panicked {
        return None;
    }

    let mut runtime_bindings = collect_runtime_bindings(ret.program.body.iter());
    let macro_imports =
        collect_artifact_macro_import_bindings(ret.program.body.iter(), nuxt_page_meta);
    for name in &macro_imports {
        runtime_bindings.remove(name);
    }
    let mut ranges = Vec::new();
    for stmt in ret.program.body.iter() {
        if is_artifact_macro_only_import(stmt, nuxt_page_meta) {
            let span = stmt.span();
            let start = span.start as usize;
            let end = span.end as usize;
            if start <= end && end <= content.len() {
                ranges.push((start, end));
            }
            continue;
        }

        let import_removals = artifact_macro_import_removal_spans(stmt, content, nuxt_page_meta);
        if !import_removals.is_empty() {
            ranges.extend(import_removals);
            continue;
        }

        let Some(call) = artifact_call_from_statement(stmt) else {
            continue;
        };
        let Some(name) = call_name(call) else {
            continue;
        };
        if runtime_bindings.contains(name)
            || (name == "definePageMeta" && !nuxt_page_meta && !macro_imports.contains(name))
        {
            continue;
        }
        if macro_artifact_kind(name).is_none() {
            continue;
        }

        let span = stmt.span();
        let start = span.start as usize;
        let end = span.end as usize;
        if start <= end && end <= content.len() {
            ranges.push((start, end));
        }
    }

    if ranges.is_empty() {
        return None;
    }
    let edits = ranges.into_iter().map(|(start, end)| (start, end, ""));
    Some(apply_edits(content, edits))
}

fn contains_artifact_macro_candidate(content: &str) -> bool {
    artifact_macro_names().any(|name| content.contains(name))
}

fn artifact_call_from_statement<'a>(stmt: &'a Statement<'a>) -> Option<&'a CallExpression<'a>> {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => unwrap_call_expression(&expr_stmt.expression),
        _ => None,
    }
}

fn unwrap_call_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::TSAsExpression(ts_as) => unwrap_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_call_expression(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(paren) => unwrap_call_expression(&paren.expression),
        _ => None,
    }
}

fn call_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    match &call.callee {
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

fn argument_source(arg: &Argument<'_>, source: &str) -> String {
    let span = arg.span();
    let start = span.start as usize;
    let end = span.end as usize;
    source
        .get(start..end)
        .map(ToCompactString::to_compact_string)
        .unwrap_or_default()
}

fn build_artifact_module(kind: &str, payload: &str, static_imports: &str) -> String {
    let mut module_code = String::default();
    module_code.push_str(static_imports);

    if kind == "nuxt.definePageMeta" {
        module_code.push_str("const __nuxt_page_meta = ");
        module_code.push_str(payload.trim());
        module_code.push_str("\nexport default __nuxt_page_meta\n");
        return module_code;
    }

    module_code.push_str("export default ");
    module_code.push_str(payload.trim());
    module_code.push('\n');
    module_code
}

#[cfg(test)]
mod tests;
