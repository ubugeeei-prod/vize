//! Attribute parsing, sorting, and rendering.
//!
//! Provides the `ParsedAttribute` type and functions for sorting attributes
//! according to Vue style guide order, plus rendering them back to strings.

use crate::options::{AttributeSortOrder, FormatOptions};
use vize_carton::{String, ToCompactString};

/// Parsed attribute with structured information for sorting and rendering.
#[derive(Debug, Clone)]
pub(crate) struct ParsedAttribute {
    /// Normalized attribute name (after shorthand conversion)
    pub(crate) name: String,
    /// Attribute value (without quotes), None for boolean attrs like `v-else`
    pub(crate) value: Option<String>,
    /// Sort priority (lower = earlier in output)
    pub(crate) priority: u8,
    /// Original index in the source for stable sorting
    pub(crate) original_index: usize,
}

/// Sort attributes based on the configured options.
pub(crate) fn sort_attributes(attrs: &mut [ParsedAttribute], options: &FormatOptions) {
    match options.attribute_sort_order {
        AttributeSortOrder::Alphabetical => {
            // Decorate-sort-undecorate: `attr_sort_key` lowercases the name,
            // so computing it inside a comparator re-allocates O(n log n)
            // times. `sort_by_cached_key` evaluates the key closure exactly
            // once per attribute and caches it, then sorts on the cached
            // tuples. The composite key `(priority, group, lowercased base,
            // original index)` reproduces the previous comparator (including
            // the stable original-index tie-break) exactly.
            let merge_bind = options.merge_bind_and_non_bind_attrs;
            attrs.sort_by_cached_key(|attr| {
                let (group, base) = attr_sort_key(&attr.name, merge_bind);
                (attr.priority, group, base, attr.original_index)
            });
        }
        AttributeSortOrder::AsWritten => {
            // Only sort by priority group, keep original order within groups
            attrs.sort_by(|a, b| {
                let cmp = a.priority.cmp(&b.priority);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
                a.original_index.cmp(&b.original_index)
            });
        }
    }
}

/// Generate a sort key for alphabetical ordering within a group.
///
/// When `merge_bind` is false, non-bind attrs come before bind attrs,
/// then each sub-group is sorted alphabetically:
///   `class`, `id`, `:class`, `:id`
///
/// When `merge_bind` is true, bind prefix is stripped so `:class` and
/// `class` are sorted together:
///   `class`, `:class`, `id`, `:id`
fn attr_sort_key(name: &str, merge_bind: bool) -> (u8, String) {
    if merge_bind {
        // Strip bind prefix for comparison
        let base = name
            .strip_prefix(':')
            .or_else(|| name.strip_prefix("v-bind:"))
            .unwrap_or(name);
        (0, base.to_ascii_lowercase().into())
    } else {
        // Non-bind first (0), then bind (1)
        let is_bind = name.starts_with(':') || name.starts_with("v-bind:");
        let base = name
            .strip_prefix(':')
            .or_else(|| name.strip_prefix("v-bind:"))
            .unwrap_or(name);
        let group = if is_bind { 1 } else { 0 };
        (group, base.to_ascii_lowercase().into())
    }
}

/// Attribute sort priority based on the Vue.js style guide:
///
/// 0. `is`
/// 1. `v-for`
/// 2. `v-if` / `v-else-if` / `v-else`
/// 3. `v-show`
/// 4. `id`
/// 5. `ref`
/// 6. `key` / `:key`
/// 7. `v-model`
/// 8. props & attributes -- both bound (`:class`) and static (`class`) share the
///    same priority so that related pairs like `class`/`:class` stay adjacent.
/// 9. events (`@xxx`)
/// 10. `v-slot` / `#xxx`
/// 11. `v-html` / `v-text`
pub(crate) fn attribute_priority(name: &str) -> u8 {
    if name == "is" || name == ":is" || name == "v-is" {
        return 0;
    }
    if name == "v-for" {
        return 1;
    }
    if name == "v-if" || name == "v-else-if" || name == "v-else" {
        return 2;
    }
    if name == "v-show" {
        return 3;
    }
    if name == "id" {
        return 4;
    }
    if name == "ref" {
        return 5;
    }
    if name == "key" || name == ":key" {
        return 6;
    }
    if name.starts_with("v-model") {
        return 7;
    }
    // Events
    if name.starts_with('@') || name.starts_with("v-on") {
        return 9;
    }
    // Slots
    if name.starts_with('#') || name.starts_with("v-slot") {
        return 10;
    }
    if name == "v-html" || name == "v-text" {
        return 11;
    }
    // Both bound props (:class, :style, :xxx) and regular attributes (class, style, xxx)
    // share the same priority so that related pairs stay adjacent.
    8
}

/// Render an attribute back to its string representation.
#[allow(clippy::disallowed_macros)]
pub(crate) fn render_attribute(attr: &ParsedAttribute) -> String {
    match &attr.value {
        Some(value) => {
            let quote = attribute_quote(value);
            let value = escape_attribute_value(value, quote);
            format!("{}={}{}{}", attr.name, quote, value, quote).into()
        }
        None => attr.name.clone(),
    }
}

fn attribute_quote(value: &str) -> char {
    if value.contains('"') && !value.contains('\'') {
        '\''
    } else {
        '"'
    }
}

fn escape_attribute_value(value: &str, quote: char) -> String {
    if !value.contains(quote) {
        return value.to_compact_string();
    }

    let mut escaped = String::default();
    for ch in value.chars() {
        match (quote, ch) {
            ('"', '"') => escaped.push_str("&quot;"),
            ('\'', '\'') => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{ParsedAttribute, render_attribute};

    #[test]
    fn render_attribute_uses_single_quotes_when_value_contains_double_quotes() {
        let attr = ParsedAttribute {
            name: "title".into(),
            value: Some(r#"say "hello""#.into()),
            priority: 0,
            original_index: 0,
        };

        assert_eq!(render_attribute(&attr).as_str(), r#"title='say "hello"'"#);
    }

    #[test]
    fn render_attribute_escapes_double_quotes_when_value_contains_both_quote_styles() {
        let attr = ParsedAttribute {
            name: "title".into(),
            value: Some(r#"say "hello" and 'bye'"#.into()),
            priority: 0,
            original_index: 0,
        };

        assert_eq!(
            render_attribute(&attr).as_str(),
            r#"title="say &quot;hello&quot; and 'bye'""#
        );
    }
}
