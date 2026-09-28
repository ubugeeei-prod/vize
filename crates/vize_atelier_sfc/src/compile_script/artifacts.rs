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

use crate::module_map::{Runs, apply_edits};
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

    let static_imports = collect_static_imports(ret.program.body.iter(), content);
    let mut runtime_bindings = collect_runtime_bindings(ret.program.body.iter());
    for name in collect_artifact_macro_import_bindings(ret.program.body.iter()) {
        runtime_bindings.remove(&name);
    }
    let mut artifacts = Vec::new();

    for stmt in ret.program.body.iter() {
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
    erase_artifact_macro_statements_traced(content).map(|(erased, _)| erased)
}

/// [`erase_artifact_macro_statements`] with the erased text's provenance in
/// `content` (Davinci P3-9 source maps).
pub(crate) fn erase_artifact_macro_statements_traced(content: &str) -> Option<(String, Runs)> {
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
    for name in collect_artifact_macro_import_bindings(ret.program.body.iter()) {
        runtime_bindings.remove(&name);
    }
    let mut ranges = Vec::new();
    for stmt in ret.program.body.iter() {
        if is_artifact_macro_only_import(stmt) {
            let span = stmt.span();
            let start = span.start as usize;
            let end = span.end as usize;
            if start <= end && end <= content.len() {
                ranges.push((start, end));
            }
            continue;
        }

        if let Some((start, end)) = artifact_macro_import_removal_span(stmt, content) {
            ranges.push((start, end));
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
        let Some(import) = content.get(start..end) else {
            continue;
        };

        if let Some((remove_start, remove_end)) = artifact_macro_import_removal_span(stmt, content)
        {
            let mut cleaned = String::default();
            cleaned.push_str(content.get(start..remove_start).unwrap_or_default());
            cleaned.push_str(content.get(remove_end..end).unwrap_or_default());
            imports.push_str(cleaned.trim());
        } else {
            imports.push_str(import.trim());
        }
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
            if let Some(local) =
                artifact_macro_import_local_name(specifier, import_decl.source.value.as_str())
            {
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
        && specifiers.iter().all(|specifier| {
            artifact_macro_import_local_name(specifier, import_decl.source.value.as_str()).is_some()
        })
}

/// Remove a compile-time macro from an import that also carries runtime bindings.
/// The source spans keep the remaining import and its provenance byte-identical.
fn artifact_macro_import_removal_span(
    stmt: &Statement<'_>,
    content: &str,
) -> Option<(usize, usize)> {
    let Statement::ImportDeclaration(import_decl) = stmt else {
        return None;
    };
    if import_decl.import_kind.is_type()
        || !is_known_artifact_macro_import_source(import_decl.source.value.as_str())
    {
        return None;
    }
    let specifiers = import_decl.specifiers.as_ref()?;
    if specifiers.len() < 2 {
        return None;
    }
    let macro_indices: Vec<_> = specifiers
        .iter()
        .enumerate()
        .filter_map(|(index, specifier)| {
            artifact_macro_import_local_name(specifier, import_decl.source.value.as_str())
                .map(|_| index)
        })
        .collect();
    if macro_indices.len() != 1 {
        return None;
    }
    let index = *macro_indices.first()?;
    let macro_span = specifiers.get(index)?.span();
    let other_named: Vec<_> = specifiers
        .iter()
        .enumerate()
        .filter(|(other, specifier)| {
            *other != index && matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(_))
        })
        .collect();
    if other_named.is_empty() {
        let before = content.get(import_decl.span.start as usize..macro_span.start as usize)?;
        let open = before.rfind('{')? + import_decl.span.start as usize;
        let after = content.get(macro_span.end as usize..import_decl.span.end as usize)?;
        let close = after.find('}')? + macro_span.end as usize + 1;
        let default_end = specifiers.first()?.span().end as usize;
        return Some((default_end.min(open), close));
    }
    if let Some(next) = other_named.iter().find(|(other, _)| *other > index) {
        return Some((macro_span.start as usize, next.1.span().start as usize));
    }
    let previous = other_named.last()?.1.span();
    Some((previous.end as usize, macro_span.end as usize))
}

fn artifact_macro_import_local_name<'a>(
    specifier: &'a ImportDeclarationSpecifier<'a>,
    source: &str,
) -> Option<&'a str> {
    let ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier else {
        return None;
    };
    if spec.import_kind.is_type() {
        return None;
    }
    let imported = spec.imported.name().as_str();
    let local = spec.local.name.as_str();
    if imported != local
        || macro_artifact_kind(imported).is_none()
        || (source == "#imports" && imported != "definePageMeta")
    {
        return None;
    }
    Some(local)
}

fn is_known_artifact_macro_import_source(source: &str) -> bool {
    matches!(source, "@typed-router" | "#imports")
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
