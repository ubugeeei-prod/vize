//! Helpers for splicing `<script setup generic="...">` type parameters into
//! hoisted type/interface declarations lifted to module scope.

use vize_carton::{String, append};
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
