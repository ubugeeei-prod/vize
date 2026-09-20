//! Resolve macro type dependencies in the same scope as their declarations.

use vize_croquis::Croquis;

use crate::virtual_ts::type_dependencies;

fn binding_is_import(summary: &Croquis, name: &str) -> bool {
    summary.binding_spans.get(name).is_some_and(|(start, end)| {
        summary
            .import_statements
            .iter()
            .any(|import| *start >= import.start && *end <= import.end)
    })
}

pub(in crate::virtual_ts) fn define_props_type_requires_setup_scope(summary: &Croquis) -> bool {
    summary
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_ref())
        .is_some_and(|args| {
            super::generics::setup_signature(summary).1.is_some()
                || macro_type_requires_setup_scope(summary, args)
        })
}

/// Imports and hoisted declarations are visible to module-level aliases.
/// Setup values (including classes/enums in type position) and dependent local
/// types must be evaluated inside setup. The AST query excludes property names,
/// comments, string literals, generic parameters, mapped keys and infer locals.
pub(super) fn macro_type_requires_setup_scope(summary: &Croquis, type_args: &str) -> bool {
    let inner = type_args
        .strip_prefix('<')
        .and_then(|text| text.strip_suffix('>'))
        .unwrap_or(type_args);
    let Some(names) = type_dependencies::free_names(inner) else {
        // Keep malformed authored type text in its original scope; the parser
        // diagnostic remains authoritative instead of inventing module errors.
        return true;
    };
    names.iter().any(|name| {
        if binding_is_import(summary, name) {
            return false;
        }
        if let Some(export) = summary
            .type_exports
            .iter()
            .find(|export| export.name == *name)
        {
            return !export.hoisted;
        }
        summary.bindings.bindings.contains_key(name)
            || summary.types.definitions().interfaces.contains_key(name)
            || summary.types.definitions().type_aliases.contains_key(name)
    })
}
