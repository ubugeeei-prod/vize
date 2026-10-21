use crate::{
    attribute::write_attr_value,
    options::{AttributeSortOrder, FormatOptions},
};
use unicode_width::UnicodeWidthStr;
use vize_l0::String;

#[cfg(test)]
use super::directives::directive_head;
use super::directives::{directive_name, should_format_expression, static_directive_argument};
use vize_l1::markup::{DirectiveName, DirectivePrefix};
mod render;
#[cfg(test)]
use render::write_rendered_attribute;
pub(crate) use render::{AttributeLayout, write_rendered_attributes};

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
    pub(crate) indent_multiline_value: bool,
    pub(crate) owns_value_lines: bool,
}

/// Sort attributes based on the configured options.
pub(crate) fn sort_attributes(attrs: &mut [ParsedAttribute], options: &FormatOptions) {
    // Bindings and directives can execute Vue instance getters, calls,
    // assignments, or updates while the render data object is constructed.
    // Keep them fixed and sort only unique, literal HTML attributes between
    // those barriers. This also subsumes the no-argument v-bind/v-on spread
    // boundary: no dynamic attribute may cross another one.
    for segment in attrs.split_mut(is_order_sensitive_attribute) {
        // Empty and singleton groups already have their only possible order.
        if segment.len() < 2 {
            continue;
        }
        if has_duplicate_static_names(segment) {
            continue;
        }
        sort_attribute_segment(segment, options);
    }
}

/// Sort one contiguous group with no order-sensitive dynamic attributes.
fn sort_attribute_segment(attrs: &mut [ParsedAttribute], options: &FormatOptions) {
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

fn is_order_sensitive_attribute(attr: &ParsedAttribute) -> bool {
    let name = attr.name.as_str();
    name.starts_with([':', '@', '#', '.'])
        || name.starts_with("v-")
        || attr
            .value
            .as_deref()
            .is_some_and(|value| value.contains("{{"))
}

fn has_duplicate_static_names(attrs: &[ParsedAttribute]) -> bool {
    attrs.iter().enumerate().any(|(index, attr)| {
        attrs
            .get(..index)
            .unwrap_or_default()
            .iter()
            .any(|previous| previous.name.eq_ignore_ascii_case(&attr.name))
    })
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

/// Attribute sort priority mirroring patina's `vue/attribute-order` groups
/// (the eslint-plugin-vue `vue/attributes-order` default), so default fmt
/// output can never introduce that lint finding (#3251). Patina quirks are
/// mirrored on purpose: bound `is`, `id`, `ref`, `key`, and Vue 2 slots keep
/// their named groups; unmatched
/// directives (`v-is`, `v-memo`) join the slots.
/// 0 `is`/`:is`; 1 `v-for`; 2 conditionals `v-if`/`v-else-if`/`v-else`/
/// `v-show`/`v-cloak`; 3 render modifiers `v-pre`/`v-once`; 4 `id`; 5 unique
/// `ref`/`key`/`slot`/`slot-scope`; 6 `v-model`; 7 other directives
/// `v-slot`/`#xxx`; 8 other attributes; 9 events `@xxx`/`v-on`; 10 content
/// `v-html`/`v-text`.
#[cfg(test)]
pub(crate) fn attribute_priority(name: &str) -> u8 {
    let head = directive_head(name);
    attribute_priority_with_head(name, head.as_ref())
}

pub(crate) fn attribute_priority_with_head(name: &str, head: Option<&DirectiveName>) -> u8 {
    if name == "is" {
        return 0;
    }
    if name == "id" {
        return 4;
    }
    if matches!(name, "ref" | "key" | "slot" | "slot-scope") {
        return 5;
    }
    let Some(head) = head else {
        return 8;
    };
    match head.prefix {
        DirectivePrefix::Bind => match static_directive_argument(name, head) {
            Some("is") => 0,
            Some("id") => 4,
            Some("ref" | "key" | "slot" | "slot-scope") => 5,
            _ => 8,
        },
        // Patina currently keeps property shorthand in OtherAttrs.
        DirectivePrefix::Prop => 8,
        DirectivePrefix::On => 9,
        DirectivePrefix::Slot => 7,
        DirectivePrefix::Full => match directive_name(name, head) {
            Some("for") => 1,
            Some("if" | "else-if" | "else" | "show" | "cloak") => 2,
            Some("pre" | "once") => 3,
            Some("model") => 6,
            Some("html" | "text") => 10,
            Some("on") => 9,
            Some("bind") => match static_directive_argument(name, head) {
                Some("is") => 0,
                Some("id") => 4,
                Some("ref" | "key" | "slot" | "slot-scope") => 5,
                _ => 8,
            },
            Some("slot") | Some(_) | None => 7,
        },
    }
}

/// Render an attribute back to its string representation.
pub(crate) fn render_attribute(attr: &ParsedAttribute) -> String {
    match &attr.value {
        Some(value) => {
            let mut rendered = String::with_capacity(attr.name.len() + value.len() + 3);
            rendered.push_str(&attr.name);
            rendered.push('=');
            if should_format_expression(&attr.name) {
                rendered.push('"');
                for (index, segment) in value.split('"').enumerate() {
                    if index != 0 {
                        rendered.push_str("&quot;");
                    }
                    rendered.push_str(segment);
                }
                rendered.push('"');
            } else {
                write_attr_value(value, |segment| rendered.push_str(segment));
            }
            rendered
        }
        None => attr.name.clone(),
    }
}

pub(crate) fn rendered_attribute_is_multiline(attr: &str, options: &FormatOptions) -> bool {
    attr.contains('\n') || (options.end_of_line == crate::EndOfLine::Cr && attr.contains('\r'))
}

pub(crate) fn should_use_multiline_attrs(
    options: &FormatOptions,
    tag_name: &str,
    attrs: &[ParsedAttribute],
    rendered: &[String],
    start_column: usize,
    closing_len: usize,
) -> bool {
    if rendered
        .iter()
        .any(|rendered| rendered_attribute_is_multiline(rendered, options))
    {
        return true;
    }

    // Count limits force a break only when there is more than one attribute.
    // A single attribute that fits stays on the tag line; one that does not
    // fit uses the same print-width check as a multi-attribute tag. (#7236)
    if attrs.len() > 1 {
        if options.single_attribute_per_line {
            return true;
        }

        if let Some(max) = options.max_attributes_per_line {
            return attrs.len() > max as usize;
        }
    }

    // Measure the printed line: the indentation including the SFC level around
    // the template, the tag, every attribute by display width, and the `>` or
    // ` />` that closes it. UTF-8 byte length never undercounts display width,
    // so only a tag whose bytes overflow needs the exact measurement.
    let print_width = options.print_width as usize;
    let fixed = start_column + 1 + closing_len;
    let bytes: usize = rendered.iter().map(|a| 1 + a.len()).sum();
    if fixed + tag_name.len() + bytes <= print_width {
        return false;
    }
    let columns: usize = rendered.iter().map(|a| 1 + a.width()).sum();
    fixed + tag_name.width() + columns > print_width
}

#[cfg(test)]
mod tests;
