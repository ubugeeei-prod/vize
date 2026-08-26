//! Helpers for resolving the component's *own* name so a recursive
//! self-reference in the template is not reported as unregistered (#4953).
//!
//! A `<script setup>` component can reference itself by its filename-derived
//! name or by the name declared via `defineOptions({ name })` — a documented
//! Vue feature for recursive components that needs no import.

use vize_croquis::Croquis;
use vize_croquis::macros::MacroKind;

/// The file name without directories or extension
/// (`"src/tree-item.vue"` -> `"tree-item"`).
pub(super) fn file_stem(filename: &str) -> &str {
    let base = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    base.split('.').next().unwrap_or(base)
}

/// The component name declared via `defineOptions({ name: "..." })`, if any.
pub(super) fn define_options_name(analysis: &Croquis) -> Option<&str> {
    let call = analysis
        .macros
        .all_calls()
        .iter()
        .find(|call| matches!(call.kind, MacroKind::DefineOptions))?;
    options_name_property(call.runtime_args.as_ref()?.as_str())
}

/// Pragmatic scan of a `defineOptions` argument source for a `name` property
/// with a plain string-literal value. Returns the literal's content.
fn options_name_property(args: &str) -> Option<&str> {
    let bytes = args.as_bytes();
    let mut search = 0usize;
    while let Some(found) = args[search..].find("name") {
        let start = search + found;
        search = start + "name".len();
        // The key may be bare (`name:`) or quoted (`"name":`); anything else
        // directly before it (`fullname:`) is a different key.
        let before = start.checked_sub(1).map(|i| bytes[i]);
        let quoted_key = matches!(before, Some(b'\'' | b'"'));
        if !matches!(before, None | Some(b'{' | b',' | b'\'' | b'"'))
            && !before.is_some_and(|b| b.is_ascii_whitespace())
        {
            continue;
        }
        let mut cursor = search;
        if quoted_key {
            if bytes.get(cursor).copied() != before {
                continue;
            }
            cursor += 1;
        }
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b':') {
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let quote = match bytes.get(cursor) {
            Some(&q @ (b'\'' | b'"' | b'`')) => q,
            _ => continue,
        };
        cursor += 1;
        let value_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != quote {
            // Stay conservative on escapes and template interpolation.
            if bytes[cursor] == b'\\' || (quote == b'`' && bytes[cursor] == b'$') {
                return None;
            }
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return None;
        }
        return Some(&args[value_start..cursor]);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{file_stem, options_name_property};

    #[test]
    fn extracts_name_property() {
        assert_eq!(
            options_name_property(r#"{ name: "TreeItem" }"#),
            Some("TreeItem")
        );
        assert_eq!(
            options_name_property(r#"{ "name": 'TreeItem' }"#),
            Some("TreeItem")
        );
        assert_eq!(
            options_name_property(r#"{ inheritAttrs: false, name: `TreeItem` }"#),
            Some("TreeItem")
        );
    }

    #[test]
    fn rejects_non_name_keys_and_dynamic_values() {
        assert_eq!(options_name_property(r#"{ fullname: "x" }"#), None);
        assert_eq!(options_name_property(r#"{ name: dynamic }"#), None);
        assert_eq!(options_name_property(r#"{ name: `a${b}` }"#), None);
        assert_eq!(options_name_property(r#"{ inheritAttrs: false }"#), None);
    }

    #[test]
    fn strips_directories_and_extension() {
        assert_eq!(file_stem("src/components/tree-item.vue"), "tree-item");
        assert_eq!(file_stem("App.vue"), "App");
        assert_eq!(file_stem(r"win\path\Item.vue"), "Item");
    }
}
