use vize_carton::{CompactString, FxHashSet, String, camelize, capitalize, cstr};
use vize_croquis::Croquis;

use crate::virtual_ts::component_reference::{
    component_reference_alias, contains_compact_name, has_type_only_component_candidate,
};

use super::super::types::VirtualTsOptions;
use super::imports::extract_declared_name;

/// Emit auto-import stubs (e.g. Nuxt composables).
///
/// Only names that are not already declared via imports or script bindings get
/// a stub. `imported_names` carries every module-level import so plain
/// `<script>` imports are covered too: `summary.bindings` only holds
/// `<script setup>` bindings when both blocks exist.
pub(super) fn emit_auto_import_stubs(
    ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
    imported_names: &FxHashSet<&str>,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
) {
    let mut has_header = false;
    for stub in &options.auto_import_stubs {
        let name = extract_declared_name(stub);
        if let Some(name) = name {
            // Skip if already imported or declared in script bindings
            if (summary.bindings.bindings.contains_key(name)
                && !contains_compact_name(syntactic_type_only_imported_names, name))
                || (imported_names.contains(&name)
                    && !contains_compact_name(syntactic_type_only_imported_names, name))
            {
                continue;
            }
        }
        if !has_header {
            ts.push_str("\n// Auto-import stubs (framework-provided globals)\n");
            has_header = true;
        }
        if let Some(name) = name
            && let Some(alias) = component_alias_for_type_only_auto_import(
                summary,
                syntactic_type_only_imported_names,
                name,
            )
            && let Some(rewritten) = rewrite_declared_name(stub, name, alias.as_str())
        {
            ts.push_str(rewritten.as_str());
        } else {
            ts.push_str(stub);
        }
        ts.push('\n');
    }
}

fn component_alias_for_type_only_auto_import(
    summary: &Croquis,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    name: &str,
) -> Option<String> {
    if !contains_compact_name(syntactic_type_only_imported_names, name) {
        return None;
    }
    summary
        .used_components
        .iter()
        .find(|component| {
            let component_name = component.as_str();
            has_type_only_component_candidate(syntactic_type_only_imported_names, component_name)
                && component_name_matches(component_name, name)
        })
        .map(|component| component_reference_alias(component.as_str()))
}

fn component_name_matches(template_name: &str, binding_name: &str) -> bool {
    let camel_name = camelize(template_name);
    let pascal_name = capitalize(camel_name.as_str());
    [template_name, camel_name.as_str(), pascal_name.as_str()].contains(&binding_name)
}

fn rewrite_declared_name(stub: &str, name: &str, alias: &str) -> Option<String> {
    for prefix in [
        "declare const ",
        "declare let ",
        "declare var ",
        "declare function ",
    ] {
        let Some(rest) = stub.strip_prefix(prefix) else {
            continue;
        };
        let Some(after_name) = rest.strip_prefix(name) else {
            continue;
        };
        if after_name.chars().next().is_some_and(|character| {
            character == '<'
                || character == '('
                || character == ':'
                || character == '='
                || character == ';'
                || character.is_ascii_whitespace()
        }) {
            return Some(cstr!("{prefix}{alias}{after_name}"));
        }
    }
    None
}
