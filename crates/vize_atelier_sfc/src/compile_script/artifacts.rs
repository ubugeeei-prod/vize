//! Compile-time macro artifact extraction.
//!
//! These helpers keep ecosystem macro output independent from any specific
//! bundler hook. The SFC compiler can erase the runtime call while still
//! returning a loadable artifact for tools such as file-based routers.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{FxHashSet, String, ToCompactString};
use vize_croquis::macros::{artifact_macro_names, macro_artifact_kind};

use crate::types::SfcMacroArtifact;

use super::runtime_bindings::collect_runtime_bindings;

pub(crate) fn extract_macro_artifacts(
    content: &str,
    absolute_offset: usize,
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

    extract_macro_artifacts_from_program(&ret.program, content, absolute_offset)
}

pub(crate) fn extract_macro_artifacts_from_program(
    program: &oxc_ast::ast::Program<'_>,
    content: &str,
    absolute_offset: usize,
) -> Vec<SfcMacroArtifact> {
    if !contains_artifact_macro_candidate(content) {
        return Vec::new();
    }
    let static_imports = collect_static_imports(program.body.iter(), content);
    let mut runtime_bindings = collect_runtime_bindings(program.body.iter());
    for name in collect_artifact_macro_import_bindings(program.body.iter()) {
        runtime_bindings.remove(&name);
    }
    let mut artifacts = Vec::new();

    for stmt in program.body.iter() {
        let Some(call) = artifact_call_from_statement(stmt) else {
            continue;
        };
        let Some(name) = call_name(call) else {
            continue;
        };
        if runtime_bindings.contains(name) {
            continue;
        }
        let Some(kind) = macro_artifact_kind(name) else {
            continue;
        };

        let start = call.span.start as usize;
        let end = call.span.end as usize;
        if start > end || end > content.len() {
            continue;
        }

        let source = (&content[start..end]).to_compact_string();
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
    if !contains_artifact_macro_candidate(content) {
        return None;
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();

    if ret.panicked {
        return None;
    }

    erase_artifact_macro_statements_from_program(&ret.program, content)
}

pub(crate) fn erase_artifact_macro_statements_from_program(
    program: &oxc_ast::ast::Program<'_>,
    content: &str,
) -> Option<String> {
    if !contains_artifact_macro_candidate(content) {
        return None;
    }
    let mut runtime_bindings = collect_runtime_bindings(program.body.iter());
    for name in collect_artifact_macro_import_bindings(program.body.iter()) {
        runtime_bindings.remove(&name);
    }
    let mut ranges = Vec::new();
    for stmt in program.body.iter() {
        if is_artifact_macro_only_import(stmt) {
            let span = stmt.span();
            let start = span.start as usize;
            let end = span.end as usize;
            if start <= end && end <= content.len() {
                ranges.push((start, end));
            }
            continue;
        }

        let Some(call) = artifact_call_from_statement(stmt) else {
            continue;
        };
        let Some(name) = call_name(call) else {
            continue;
        };
        if runtime_bindings.contains(name) {
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

    let mut erased = String::with_capacity(content.len());
    let mut cursor = 0usize;
    for (start, end) in ranges {
        if start < cursor {
            continue;
        }
        erased.push_str(&content[cursor..start]);
        cursor = end;
    }
    erased.push_str(&content[cursor..]);
    Some(erased)
}

pub(crate) fn contains_artifact_macro_candidate(content: &str) -> bool {
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
    if start > end || end > source.len() {
        return String::default();
    }
    (&source[start..end]).to_compact_string()
}

fn collect_static_imports<'a>(
    statements: impl Iterator<Item = &'a Statement<'a>>,
    content: &str,
) -> String {
    let mut imports = String::default();

    for stmt in statements {
        if !matches!(stmt, Statement::ImportDeclaration(_)) {
            continue;
        }
        if is_artifact_macro_only_import(stmt) {
            continue;
        }

        let span = stmt.span();
        let start = span.start as usize;
        let end = span.end as usize;
        if start > end || end > content.len() {
            continue;
        }

        imports.push_str(content[start..end].trim());
        imports.push('\n');
    }

    imports
}

fn collect_artifact_macro_import_bindings<'a>(
    statements: impl Iterator<Item = &'a Statement<'a>>,
) -> FxHashSet<String> {
    let mut bindings = FxHashSet::default();

    for stmt in statements {
        let Statement::ImportDeclaration(import_decl) = stmt else {
            continue;
        };
        if import_decl.import_kind.is_type()
            || !is_known_artifact_macro_import_source(import_decl.source.value.as_str())
        {
            continue;
        }
        let Some(specifiers) = import_decl.specifiers.as_ref() else {
            continue;
        };
        for specifier in specifiers {
            if let Some(local) = artifact_macro_import_local_name(specifier) {
                bindings.insert(local.into());
            }
        }
    }

    bindings
}

fn is_artifact_macro_only_import(stmt: &Statement<'_>) -> bool {
    let Statement::ImportDeclaration(import_decl) = stmt else {
        return false;
    };
    if import_decl.import_kind.is_type()
        || !is_known_artifact_macro_import_source(import_decl.source.value.as_str())
    {
        return false;
    }
    let Some(specifiers) = import_decl.specifiers.as_ref() else {
        return false;
    };
    !specifiers.is_empty()
        && specifiers
            .iter()
            .all(|specifier| artifact_macro_import_local_name(specifier).is_some())
}

fn artifact_macro_import_local_name<'a>(
    specifier: &'a ImportDeclarationSpecifier<'a>,
) -> Option<&'a str> {
    let ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier else {
        return None;
    };
    if spec.import_kind.is_type() {
        return None;
    }
    let imported = spec.imported.name().as_str();
    let local = spec.local.name.as_str();
    if imported != local || macro_artifact_kind(imported).is_none() {
        return None;
    }
    Some(local)
}

fn is_known_artifact_macro_import_source(source: &str) -> bool {
    matches!(source, "@typed-router")
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
