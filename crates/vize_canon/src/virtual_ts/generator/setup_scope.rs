//! Whether a macro's type argument can be resolved from module scope.
//!
//! Script content lands inside the synthetic `__setup` function, so a
//! module-scope alias (`export type Props`, `export type Emits`) only resolves
//! names that are hoisted out of it: imports and hoisted type exports. A type
//! argument that reaches for a setup value (`typeof state`) or a locally
//! declared, non-hoisted type is invisible from there.

use vize_carton::{FxHashSet, String};
use vize_croquis::Croquis;

use super::generics::{is_ident_byte, references_any_identifier, skip_ascii_ws};

fn is_identifier_start_byte(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphabetic()
}

fn collect_typeof_root_identifiers(source: &str) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut idents = Vec::new();
    let mut from = 0usize;

    while let Some(rel) = source[from..].find("typeof") {
        let at = from + rel;
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after_keyword = at + "typeof".len();
        let after_ok = after_keyword >= bytes.len() || !is_ident_byte(bytes[after_keyword]);
        if !before_ok || !after_ok {
            from = after_keyword;
            continue;
        }

        let ident_start = skip_ascii_ws(bytes, after_keyword);
        if ident_start >= bytes.len() || !is_identifier_start_byte(bytes[ident_start]) {
            from = after_keyword;
            continue;
        }

        let mut ident_end = ident_start + 1;
        while ident_end < bytes.len() && is_ident_byte(bytes[ident_end]) {
            ident_end += 1;
        }

        let ident = &source[ident_start..ident_end];
        if ident != "import" {
            idents.push(ident);
        }
        from = ident_end;
    }

    idents
}

fn binding_is_import(summary: &Croquis, name: &str) -> bool {
    summary.binding_spans.get(name).is_some_and(|(start, end)| {
        summary
            .import_statements
            .iter()
            .any(|imp| *start >= imp.start && *end <= imp.end)
    })
}

fn is_setup_value_binding(summary: &Croquis, name: &str) -> bool {
    summary.bindings.bindings.contains_key(name) && !binding_is_import(summary, name)
}

fn references_setup_value(summary: &Croquis, inner_type: &str) -> bool {
    collect_typeof_root_identifiers(inner_type)
        .into_iter()
        .any(|name| is_setup_value_binding(summary, name))
}

/// Type names the SFC declares that stay inside `__setup`: every locally
/// declared interface or alias except the ones hoisted to module scope.
fn setup_scoped_type_names(summary: &Croquis) -> Vec<String> {
    let hoisted: FxHashSet<&str> = summary
        .type_exports
        .iter()
        .filter(|te| te.hoisted)
        .map(|te| te.name.as_str())
        .collect();
    let definitions = summary.types.definitions();
    definitions
        .interfaces
        .keys()
        .chain(definitions.type_aliases.keys())
        .map(|name| name.as_str())
        .filter(|name| !hoisted.contains(name))
        .map(String::from)
        .collect()
}

pub(super) fn define_props_type_requires_setup_scope(summary: &Croquis) -> bool {
    let Some(type_args) = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref())
    else {
        return false;
    };
    let inner_type = inner_type_of(type_args.as_str());

    if references_setup_value(summary, inner_type) {
        return true;
    }

    let non_hoisted_type_names: Vec<String> = summary
        .type_exports
        .iter()
        .filter(|te| !te.hoisted)
        .map(|te| te.name.as_str().into())
        .collect();
    !non_hoisted_type_names.is_empty()
        && references_any_identifier(inner_type, &non_hoisted_type_names)
}

/// Whether a macro type argument only resolves inside `__setup`, so mapping a
/// module-scope alias built from it back onto the authored macro would report
/// synthetic "cannot find name" diagnostics on valid SFC source (#4074).
pub(super) fn macro_type_requires_setup_scope(summary: &Croquis, type_args: &str) -> bool {
    let inner_type = inner_type_of(type_args);
    if references_setup_value(summary, inner_type) {
        return true;
    }
    let setup_scoped = setup_scoped_type_names(summary);
    !setup_scoped.is_empty() && references_any_identifier(inner_type, &setup_scoped)
}

fn inner_type_of(type_args: &str) -> &str {
    type_args
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(type_args)
}
