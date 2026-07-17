//! Import-name extraction and `/// <reference types>` directives.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    IdentifierReference, ImportDeclarationSpecifier, ImportOrExportKind, Statement, TSTypeName,
    TSTypeQueryExprName, TSTypeReference,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::{CompactString, FxHashSet, String};
use vize_croquis::Croquis;

pub(super) fn emit_reference_path_directives(ts: &mut String, paths: &[String]) {
    let mut seen = FxHashSet::default();
    for path in paths {
        if path.is_empty() || path.contains(['\n', '\r']) || !seen.insert(path.as_str()) {
            continue;
        }
        ts.push_str("/// <reference path=\"");
        for character in path.chars() {
            match character {
                '&' => ts.push_str("&amp;"),
                '"' => ts.push_str("&quot;"),
                '<' => ts.push_str("&lt;"),
                '>' => ts.push_str("&gt;"),
                _ => ts.push(character),
            }
        }
        ts.push_str("\" />\n");
    }
}

pub(super) fn emit_reference_type_directives(
    ts: &mut String,
    script_content: Option<&str>,
) -> bool {
    let Some(script) = script_content else {
        return false;
    };

    let mut seen = FxHashSet::default();
    for line in script.lines() {
        let Some(package) = reference_types_attribute(line) else {
            continue;
        };
        if seen.insert(package) {
            append!(*ts, "/// <reference types=\"{package}\" />\n");
        }
    }
    !seen.is_empty()
}

fn reference_types_attribute(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if !line.starts_with("///") || !line.contains("<reference") {
        return None;
    }
    attribute_value(line, "types")
}

fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let needle = cstr!("{name}=");
    let start = line.find(needle.as_str())? + needle.len();
    let quote = line[start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + quote.len_utf8();
    let value_end = line[value_start..].find(quote)? + value_start;
    line.get(value_start..value_end)
}

pub(super) fn collect_imported_names<'a>(
    summary: &Croquis,
    script_content: Option<&'a str>,
) -> FxHashSet<&'a str> {
    let Some(script) = script_content else {
        return FxHashSet::default();
    };

    summary
        .import_statements
        .iter()
        .flat_map(|imp| {
            let text = script
                .get(imp.start as usize..imp.end as usize)
                .unwrap_or("");
            extract_import_names(text)
        })
        .collect()
}

pub(super) fn collect_type_only_imported_names(
    summary: &Croquis,
    script_content: Option<&str>,
) -> FxHashSet<CompactString> {
    let Some(script) = script_content else {
        return FxHashSet::default();
    };
    let import_names = collect_value_import_binding_names(summary, script);
    if import_names.is_empty() {
        return FxHashSet::default();
    }

    let usage = collect_identifier_usage(script);
    import_names
        .into_iter()
        .filter(|name| usage.type_refs.contains(name) && !usage.value_refs.contains(name))
        .collect()
}

pub(super) fn collect_setup_binding_anchor_names<'a>(
    summary: &'a Croquis,
    script_content: Option<&str>,
    template_referenced_names: Option<&FxHashSet<String>>,
) -> Vec<&'a str> {
    let type_only_imported_names = collect_type_only_imported_names(summary, script_content);
    let const_enum_names = script_content.map(super::script_module::collect_const_enum_names);
    let mut template_value_names: FxHashSet<&str> = summary
        .used_components
        .iter()
        .map(|name| name.as_str())
        .collect();
    if let Some(names) = template_referenced_names {
        template_value_names.extend(names.iter().map(|name| name.as_str()));
    }

    let mut binding_names: Vec<&str> = if let Some(names) = template_referenced_names {
        summary
            .bindings
            .bindings
            .keys()
            .map(|name| name.as_str())
            .filter(|name| {
                names
                    .iter()
                    .any(|template_name| template_name.as_str() == *name)
            })
            .collect()
    } else {
        summary
            .bindings
            .bindings
            .keys()
            .map(|name| name.as_str())
            .collect()
    };
    binding_names.retain(|name| {
        const_enum_names
            .as_ref()
            .is_none_or(|names| !contains_compact_name(names, name))
            && (!contains_compact_name(&type_only_imported_names, name)
                || template_value_names.contains(name))
    });
    binding_names.sort_unstable();
    binding_names
}

fn collect_value_import_binding_names(summary: &Croquis, script: &str) -> FxHashSet<CompactString> {
    summary
        .import_statements
        .iter()
        .flat_map(|imp| {
            let text = script
                .get(imp.start as usize..imp.end as usize)
                .unwrap_or("");
            extract_import_names(text)
                .into_iter()
                .map(CompactString::new)
                .collect::<Vec<_>>()
        })
        .filter(|name| summary.bindings.bindings.contains_key(name))
        .collect()
}

fn contains_compact_name(names: &FxHashSet<CompactString>, name: &str) -> bool {
    names.iter().any(|candidate| candidate.as_str() == name)
}

#[derive(Default)]
struct IdentifierUsage {
    type_refs: FxHashSet<CompactString>,
    value_refs: FxHashSet<CompactString>,
    type_depth: u32,
}

impl<'a> Visit<'a> for IdentifierUsage {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if self.type_depth == 0 {
            self.value_refs
                .insert(CompactString::new(ident.name.as_str()));
        }
    }

    fn visit_ts_type_reference(&mut self, ty: &TSTypeReference<'a>) {
        record_type_name_root(&ty.type_name, &mut self.type_refs);
        self.type_depth += 1;
        walk::walk_ts_type_reference(self, ty);
        self.type_depth -= 1;
    }

    fn visit_ts_type_query_expr_name(&mut self, name: &TSTypeQueryExprName<'a>) {
        record_type_query_root(name, &mut self.value_refs);
        walk::walk_ts_type_query_expr_name(self, name);
    }
}

fn collect_identifier_usage(script: &str) -> IdentifierUsage {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    let mut usage = IdentifierUsage::default();
    usage.visit_program(&parsed.program);
    usage
}

fn record_type_name_root(name: &TSTypeName<'_>, refs: &mut FxHashSet<CompactString>) {
    match name {
        TSTypeName::IdentifierReference(ident) => {
            refs.insert(CompactString::new(ident.name.as_str()));
        }
        TSTypeName::QualifiedName(qualified) => record_type_name_root(&qualified.left, refs),
        TSTypeName::ThisExpression(_) => {}
    }
}

fn record_type_query_root(name: &TSTypeQueryExprName<'_>, refs: &mut FxHashSet<CompactString>) {
    match name {
        TSTypeQueryExprName::IdentifierReference(ident) => {
            refs.insert(CompactString::new(ident.name.as_str()));
        }
        TSTypeQueryExprName::QualifiedName(qualified) => {
            record_type_name_root(&qualified.left, refs);
        }
        TSTypeQueryExprName::TSImportType(_) => {}
        _ => {}
    }
}

/// Extract imported identifier names from an import statement string.
/// Handles `import { a, b as c } from "..."` and `import D from "..."`.
/// Returns the local names (e.g., `["a", "c", "D"]`).
pub(super) fn extract_declared_name(stub: &str) -> Option<&str> {
    for prefix in [
        "declare function ",
        "declare const ",
        "declare let ",
        "declare var ",
    ] {
        let Some(rest) = stub.strip_prefix(prefix) else {
            continue;
        };
        let end = rest
            .find(['<', '(', ':', '=', ';', ' '])
            .unwrap_or(rest.len());
        let name = rest[..end].trim();
        if !name.is_empty() {
            return Some(name);
        }
    }

    None
}

fn extract_import_names(import_text: &str) -> Vec<&str> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, import_text, SourceType::ts()).parse();
    let Some(Statement::ImportDeclaration(declaration)) = parsed.program.body.first() else {
        return Vec::new();
    };
    if declaration.import_kind == ImportOrExportKind::Type {
        return Vec::new();
    }

    declaration
        .specifiers
        .iter()
        .flatten()
        .filter_map(|specifier| {
            let span = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if specifier.import_kind == ImportOrExportKind::Value =>
                {
                    specifier.local.span
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    specifier.local.span
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    specifier.local.span
                }
                ImportDeclarationSpecifier::ImportSpecifier(_) => return None,
            };
            import_text.get(span.start as usize..span.end as usize)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::extract_import_names;

    #[test]
    fn mixed_default_and_named_imports_keep_every_value_binding() {
        assert_eq!(
            extract_import_names("import Badge, { helper as local, type Props } from 'pkg'"),
            ["Badge", "local"]
        );
    }

    #[test]
    fn mixed_default_and_namespace_imports_keep_both_bindings() {
        assert_eq!(
            extract_import_names("import Badge, * as badgeNs from 'pkg'"),
            ["Badge", "badgeNs"]
        );
        assert!(extract_import_names("import type Badge from 'pkg'").is_empty());
    }
}
