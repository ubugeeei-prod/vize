//! Helpers for splicing `<script setup generic="...">` type parameters into
//! hoisted type/interface declarations lifted to module scope.

use std::collections::VecDeque;

use vize_carton::{CompactString, FxHashMap, FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, ScopeData, ScopeKind};

use crate::text_scan::{byte_before, starts_with_at};
use crate::virtual_ts::generic_parameters::GenericParameters;

pub(super) fn setup_signature(summary: &Croquis) -> (Option<&str>, Option<&str>, bool) {
    summary
        .scopes
        .iter()
        .find(|scope| matches!(scope.kind, ScopeKind::ScriptSetup))
        .and_then(|scope| match scope.data() {
            ScopeData::ScriptSetup(data) => Some((
                data.generic.as_deref(),
                data.generic
                    .as_deref()
                    .filter(|value| !value.trim().is_empty()),
                data.is_async,
            )),
            _ => None,
        })
        .unwrap_or((None, None, false))
}

pub(super) fn generic_suffix(parameters: &str) -> String {
    if parameters.trim().is_empty() {
        String::default()
    } else {
        cstr!("<{parameters}>")
    }
}

pub(super) struct HoistedGenericAliases {
    generic_decl: String,
    generic_names: String,
    type_names: Vec<String>,
    dependent_declarations: Vec<String>,
}

impl HoistedGenericAliases {
    pub(super) fn collect(
        summary: &Croquis,
        script: Option<&str>,
        generic_param: Option<&GenericParameters>,
    ) -> Option<Self> {
        let (script, generic_param) = script.zip(generic_param)?;
        let generic_names = generic_param.names.clone();
        let names: Vec<String> = generic_names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(String::from)
            .collect();
        let candidates: Vec<_> = summary
            .type_exports
            .iter()
            .filter(|export| export.hoisted)
            .filter_map(|export| {
                let source = script.get(export.start as usize..export.end as usize)?;
                let (declaration, references) =
                    crate::virtual_ts::type_dependencies::local_type_declaration(source)?;
                Some((export, declaration, references))
            })
            .collect();
        let type_names: Vec<String> = candidates
            .iter()
            .filter(|(export, source, references)| {
                names.iter().any(|name| references.contains(name.as_str()))
                    && generic_injection_point(source, export.name.as_str()).is_some()
            })
            .map(|(export, _, _)| String::from(export.name.as_str()))
            .collect();

        if type_names.is_empty() {
            return None;
        }

        // A module-level `Alias = Value` would instantiate Value's fallback
        // parameters before setup can bind T. Retain the original dependent
        // declaration inside setup instead, preserving its own generic scope.
        let mut dependents: FxHashMap<CompactString, Vec<CompactString>> = FxHashMap::default();
        for (export, _, references) in &candidates {
            for reference in references {
                dependents
                    .entry(reference.clone())
                    .or_default()
                    .push(export.name.clone());
            }
        }
        let mut reached: FxHashSet<CompactString> = type_names
            .iter()
            .map(|name| CompactString::from(name.as_str()))
            .collect();
        let mut queue: VecDeque<_> = reached.iter().cloned().collect();
        while let Some(name) = queue.pop_front() {
            for dependent in dependents.get(&name).into_iter().flatten() {
                if reached.insert(dependent.clone()) {
                    queue.push_back(dependent.clone());
                }
            }
        }
        let dependent_declarations = candidates
            .iter()
            .filter(|(export, _, _)| {
                reached.contains(&export.name)
                    && !type_names
                        .iter()
                        .any(|name| name.as_str() == export.name.as_str())
            })
            .map(|(_, source, _)| String::from(*source))
            .collect();

        Some(Self {
            generic_decl: generic_param.alias_declaration.clone(),
            generic_names,
            type_names,
            dependent_declarations,
        })
    }

    pub(super) fn emit_module_aliases(&self, ts: &mut String) {
        for (index, type_name) in self.type_names.iter().enumerate() {
            append!(
                *ts,
                "type __VizeHoistedGeneric{index}<{}> = {type_name}<{}>;\n",
                self.generic_decl,
                self.generic_names
            );
        }
    }

    pub(super) fn emit_setup_aliases(&self, ts: &mut String) {
        for (index, type_name) in self.type_names.iter().enumerate() {
            append!(
                *ts,
                "  type {type_name} = __VizeHoistedGeneric{index}<{}>;\n",
                self.generic_names
            );
        }
        for declaration in &self.dependent_declarations {
            append!(*ts, "  {declaration}\n");
        }
    }
}

/// Generic suffix (`<T extends string = any>`) for a module-scope type alias
/// whose body references an SFC generic parameter. Empty when the alias
/// references none, so non-generic aliases keep their exact shape. Every
/// parameter carries a default (added by the caller), so bare references to
/// the alias elsewhere in the module stay valid (#3065).
pub(super) fn module_alias_generic_suffix(
    generic_injection: Option<&(String, Vec<String>)>,
    inner_type: &str,
) -> String {
    generic_injection
        .filter(|(_, names)| references_any_identifier(inner_type, names))
        .map(|(defaults, _)| cstr!("<{defaults}>"))
        .unwrap_or_default()
}

/// Fallback type arguments instantiating a generic SFC's exported `Props` for
/// the non-generic component constructor: each parameter's constraint when it
/// is self-contained, `any` when the constraint names a sibling parameter (the
/// name would dangle outside the parameter list), and `unknown` when
/// unconstrained (keeps extracted component props inferable — #2701).
#[cfg(test)]
fn generic_fallback_args(generic_decl: &str) -> String {
    GenericParameters::parse(generic_decl).fallback
}

pub(super) fn is_ident_byte(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphanumeric()
}

pub(super) fn skip_ascii_ws(bytes: &[u8], mut i: usize) -> usize {
    while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
        i += 1;
    }
    i
}

/// Whether `haystack` mentions any of `idents` as a whole-word identifier.
/// Used to decide whether a lifted type declaration depends on an SFC generic
/// parameter and therefore needs that parameter re-declared on it.
pub(crate) fn references_any_identifier(haystack: &str, idents: &[String]) -> bool {
    let bytes = haystack.as_bytes();
    idents.iter().any(|ident| {
        let ident = ident.as_str();
        if ident.is_empty() {
            return false;
        }
        let mut from = 0;
        while let Some(rel) = haystack.get(from..).and_then(|rest| rest.find(ident)) {
            let at = from + rel;
            let before_ok = !byte_before(bytes, at).is_some_and(is_ident_byte);
            let after = at + ident.len();
            let after_ok = !bytes.get(after).is_some_and(|&b| is_ident_byte(b));
            if before_ok && after_ok {
                return true;
            }
            from = at + ident.len();
        }
        false
    })
}

/// Byte offset within a hoisted `type` / `interface` declaration immediately
/// after the declared name, where synthetic generic parameters can be spliced
/// in. Returns `None` if the declaration already has its own `<...>` parameter
/// list or the name can't be located (the declaration is then emitted as-is).
pub(super) fn generic_injection_point(decl: &str, type_name: &str) -> Option<usize> {
    let bytes = decl.as_bytes();
    let mut i = skip_ascii_ws(bytes, 0);

    // Optional `export` modifier.
    if starts_with_at(decl, i, "export")
        && matches!(bytes.get(i + 6), Some(b) if b.is_ascii_whitespace())
    {
        i = skip_ascii_ws(bytes, i + 6);
    }

    // Declaration keyword.
    if starts_with_at(decl, i, "type")
        && matches!(bytes.get(i + 4), Some(b) if b.is_ascii_whitespace())
    {
        i += 4;
    } else if starts_with_at(decl, i, "interface")
        && matches!(bytes.get(i + 9), Some(b) if b.is_ascii_whitespace())
    {
        i += 9;
    } else {
        return None;
    }
    i = skip_ascii_ws(bytes, i);

    // Declared name.
    if !starts_with_at(decl, i, type_name) {
        return None;
    }
    let name_end = i + type_name.len();
    // Reject partial-name matches (`Foo` inside `Foobar`).
    if matches!(bytes.get(name_end), Some(&b) if is_ident_byte(b)) {
        return None;
    }
    // Skip declarations that already declare their own type parameters.
    if bytes.get(skip_ascii_ws(bytes, name_end)) == Some(&b'<') {
        return None;
    }
    Some(name_end)
}

pub(in crate::virtual_ts) fn generic_injection(
    generic_param: Option<&GenericParameters>,
) -> Option<(String, Vec<String>)> {
    generic_param.map(|g| {
        let defaults = g.alias_declaration.clone();
        let names = g.parameter_names().map(String::from).collect();
        (defaults, names)
    })
}

#[cfg(test)]
mod tests {
    use super::{HoistedGenericAliases, generic_fallback_args, module_alias_generic_suffix};
    use vize_carton::String;

    #[test]
    fn setup_keeps_transitive_local_types_without_capturing_shadowed_parameters() {
        let script = "import type { External } from './types';\ntype Value = T extends 'number' ? number : string;\nexport type Alias = Value;\nexport interface Payload { value: Alias }\ntype Shadow<Value> = Value;\ntype Imported = External;";
        let mut analyzer =
            vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
        analyzer.analyze_script_setup_with_generic(script, Some("T extends 'number' | 'text'"));
        let summary = analyzer.finish();
        let generic = super::GenericParameters::parse("T extends 'number' | 'text'");
        let aliases = HoistedGenericAliases::collect(&summary, Some(script), Some(&generic))
            .expect("Value captures the SFC parameter");
        let mut setup = String::default();
        aliases.emit_setup_aliases(&mut setup);
        assert_eq!(
            setup.as_str(),
            concat!(
                "  type Value = __VizeHoistedGeneric0<T>;\n",
                "  type Alias = Value;\n",
                "  interface Payload { value: Alias }\n",
            )
        );
    }

    #[test]
    fn fallback_args_use_constraints_and_keep_unconstrained_params_inferable() {
        assert_eq!(
            generic_fallback_args("T extends string = 'a'").as_str(),
            "string"
        );
        assert_eq!(
            generic_fallback_args("TValue = undefined").as_str(),
            "unknown"
        );
        assert_eq!(generic_fallback_args("T = any").as_str(), "unknown");
        assert_eq!(
            generic_fallback_args("const T extends Record<string, any> = any").as_str(),
            "Record<string, any>"
        );
        assert_eq!(
            generic_fallback_args("Mode extends 'row' | 'cell' = any, TEvent extends Mode = any")
                .as_str(),
            "'row' | 'cell', any"
        );
    }

    #[test]
    fn module_alias_suffix_is_added_only_when_the_alias_references_a_generic() {
        let injection = (
            String::from("T extends string = any"),
            vec![String::from("T")],
        );
        assert_eq!(
            module_alias_generic_suffix(Some(&injection), "{ value: T | undefined }").as_str(),
            "<T extends string = any>"
        );
        assert_eq!(
            module_alias_generic_suffix(Some(&injection), "{ value: Tag }").as_str(),
            ""
        );
        assert_eq!(
            module_alias_generic_suffix(None, "{ value: T }").as_str(),
            ""
        );
    }
}
