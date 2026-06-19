//! Import-name extraction, `/// <reference types>` directives, and global
//! component stub emission for the virtual TypeScript generator.

use oxc_allocator::Allocator;
use oxc_ast::ast::{IdentifierReference, TSTypeName, TSTypeQueryExprName, TSTypeReference};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_croquis::Croquis;

use crate::virtual_ts::helpers::to_safe_identifier;
use crate::virtual_ts::types::VirtualTsOptions;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::{CompactString, FxHashSet, String, camelize, capitalize};

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

pub(super) fn emit_global_component_stubs(
    ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
    imported_names: &FxHashSet<&str>,
    enabled: bool,
) {
    if !enabled || summary.component_usages.is_empty() {
        return;
    }

    let external_template_bindings = options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect::<FxHashSet<_>>();
    let auto_import_stub_names = options
        .auto_import_stubs
        .iter()
        .filter_map(|stub| extract_declared_name(stub))
        .collect::<FxHashSet<_>>();

    let mut emitted_refs = FxHashSet::default();
    let mut has_header = false;
    for usage in &summary.component_usages {
        let name = usage.name.as_str();
        if summary.bindings.bindings.contains_key(name)
            || imported_names.contains(&name)
            || external_template_bindings.contains(&name)
            || auto_import_stub_names.contains(&name)
        {
            continue;
        }

        let component_ref = to_safe_identifier(name);
        if !emitted_refs.insert(component_ref.clone()) {
            continue;
        }

        if !has_header {
            ts.push_str("\n// Global component stubs (vue module augmentations)\n");
            has_header = true;
        }

        let pascal_name = capitalize(camelize(name).as_str());
        append!(
            *ts,
            "declare const {component_ref}: \"{name}\" extends keyof import(\"vue\").GlobalComponents ? import(\"vue\").GlobalComponents[\"{name}\"]"
        );
        if pascal_name.as_str() == name {
            ts.push_str(" : any;\n");
        } else {
            append!(
                *ts,
                " : \"{pascal_name}\" extends keyof import(\"vue\").GlobalComponents ? import(\"vue\").GlobalComponents[\"{pascal_name}\"] : any;\n"
            );
        }
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
    let mut names = Vec::new();

    if let Some(brace_start) = import_text.find('{') {
        if let Some(brace_end) = import_text.find('}') {
            let inner = &import_text[brace_start + 1..brace_end];
            for part in inner.split(',') {
                let part = part.trim();
                if part.is_empty() || part.starts_with("//") || part.starts_with("type ") {
                    continue;
                }
                if let Some(as_pos) = part.find(" as ") {
                    let alias = part[as_pos + 4..].trim();
                    if !alias.is_empty() {
                        names.push(alias);
                    }
                } else {
                    let name = part.strip_suffix('\r').unwrap_or(part);
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        names.push(name);
                    }
                }
            }
        }
    } else {
        // Handle `import Name from "..."`
        let text = import_text.trim();
        if let Some(rest) = text.strip_prefix("import ")
            && let Some(from_pos) = rest.find(" from ")
        {
            let name = rest[..from_pos].trim();
            if !name.is_empty()
                && !name.contains('{')
                && !name.contains('*')
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                names.push(name);
            }
        }
    }

    names
}
