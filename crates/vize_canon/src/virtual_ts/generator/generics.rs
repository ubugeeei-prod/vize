//! Helpers for splicing `<script setup generic="...">` type parameters into
//! hoisted type/interface declarations lifted to module scope.

use vize_carton::{String, append, cstr};
use vize_croquis::Croquis;

use crate::virtual_ts::props::{
    add_generic_defaults, extract_generic_names, strip_const_modifiers,
};

pub(super) struct HoistedGenericAliases {
    generic_decl: String,
    generic_names: String,
    type_names: Vec<String>,
}

impl HoistedGenericAliases {
    pub(super) fn collect(
        summary: &Croquis,
        script: Option<&str>,
        generic_param: Option<&str>,
    ) -> Option<Self> {
        let (script, generic_param) = script.zip(generic_param)?;
        let generic_names = extract_generic_names(generic_param);
        let names: Vec<String> = generic_names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(String::from)
            .collect();
        let type_names: Vec<String> = summary
            .type_exports
            .iter()
            .filter(|export| export.hoisted)
            .filter_map(|export| {
                let source = &script[export.start as usize..export.end as usize];
                (references_any_identifier(source, &names)
                    && generic_injection_point(source, export.name.as_str()).is_some())
                .then(|| String::from(export.name.as_str()))
            })
            .collect();

        if type_names.is_empty() {
            return None;
        }

        Some(Self {
            generic_decl: strip_const_modifiers(&add_generic_defaults(generic_param)),
            generic_names,
            type_names,
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
pub(super) fn generic_fallback_args(generic_decl: &str) -> String {
    let params = split_generic_params(generic_decl);
    let names: Vec<String> = extract_generic_names(generic_decl)
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(String::from)
        .collect();
    let mut args = String::default();
    for param in params {
        if param.trim().is_empty() {
            continue;
        }
        let arg = match param_constraint(param) {
            Some(constraint) if references_any_identifier(constraint, &names) => "any",
            Some(constraint) => constraint,
            None => "unknown",
        };
        if !args.is_empty() {
            args.push_str(", ");
        }
        args.push_str(arg);
    }
    args
}

/// Split a generic parameter list at depth-0 commas, tracking `<>` nesting
/// (the same convention as the sibling parameter helpers in `props`).
pub(super) fn split_generic_params(list: &str) -> Vec<&str> {
    let mut params = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, b) in list.bytes().enumerate() {
        match b {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b',' if depth == 0 => {
                params.push(&list[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    params.push(&list[start..]);
    params
}

/// Constraint text of one generic parameter declaration (`T extends X = D`
/// -> `X`); `None` when the parameter is unconstrained. `const` modifiers and
/// parameter defaults are excluded; `=>` arrows never terminate the
/// constraint.
fn param_constraint(param: &str) -> Option<&str> {
    let bytes = param.as_bytes();
    let mut depth = 0i32;
    let mut start = None;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b'=' if bytes.get(i + 1) == Some(&b'>') => i += 1,
            b'=' if depth == 0 => break,
            b'e' if depth == 0
                && start.is_none()
                && param[i..].starts_with("extends")
                && (i == 0 || !is_ident_byte(bytes[i - 1]))
                && !matches!(bytes.get(i + 7), Some(&b) if is_ident_byte(b)) =>
            {
                start = Some(i + 7);
                i += 6;
            }
            _ => {}
        }
        i += 1;
    }
    let constraint = param[start?..i.min(param.len())].trim();
    (!constraint.is_empty()).then_some(constraint)
}

pub(super) fn is_ident_byte(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphanumeric()
}

pub(super) fn skip_ascii_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// Whether `haystack` mentions any of `idents` as a whole-word identifier.
/// Used to decide whether a lifted type declaration depends on an SFC generic
/// parameter and therefore needs that parameter re-declared on it.
pub(super) fn references_any_identifier(haystack: &str, idents: &[String]) -> bool {
    let bytes = haystack.as_bytes();
    idents.iter().any(|ident| {
        let ident = ident.as_str();
        if ident.is_empty() {
            return false;
        }
        let mut from = 0;
        while let Some(rel) = haystack[from..].find(ident) {
            let at = from + rel;
            let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
            let after = at + ident.len();
            let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
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
    if decl[i..].starts_with("export")
        && matches!(bytes.get(i + 6), Some(b) if b.is_ascii_whitespace())
    {
        i = skip_ascii_ws(bytes, i + 6);
    }

    // Declaration keyword.
    if decl[i..].starts_with("type")
        && matches!(bytes.get(i + 4), Some(b) if b.is_ascii_whitespace())
    {
        i += 4;
    } else if decl[i..].starts_with("interface")
        && matches!(bytes.get(i + 9), Some(b) if b.is_ascii_whitespace())
    {
        i += 9;
    } else {
        return None;
    }
    i = skip_ascii_ws(bytes, i);

    // Declared name.
    if !decl[i..].starts_with(type_name) {
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

#[cfg(test)]
mod tests {
    use super::{generic_fallback_args, module_alias_generic_suffix};
    use vize_carton::String;

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
