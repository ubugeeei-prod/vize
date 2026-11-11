use crate::{
    attribute::write_attr_value,
    options::{AttributeSortOrder, FormatOptions},
};
use unicode_width::UnicodeWidthStr;
use vize_l0::String;

#[cfg(test)]
use super::directives::directive_head;
use super::directives::{should_format_expression, static_directive_argument};
use vize_l1::dialect::vue3::DirectiveKind;
use vize_l1::markup::{ArgSyntax, DirectiveName, DirectivePrefix};
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
    pub(crate) priority: usize,
    /// Original index in the source for stable sorting
    pub(crate) original_index: usize,
    pub(crate) indent_multiline_value: bool,
    pub(crate) owns_value_lines: bool,
    /// Whether the attribute bounds a sortable run, decided once from the
    /// directive head parsed during normalization.
    pub(crate) order_sensitive: bool,
}

/// Sort attributes based on the configured options.
pub(crate) fn sort_attributes(
    tag_name: &str,
    attrs: &mut [ParsedAttribute],
    options: &FormatOptions,
) {
    // Runtime bindings and custom directive hooks can observe authored order,
    // while a bare `v-bind`/`v-on` object or a dynamic argument can also merge
    // with its neighbors. Keep each boundary fixed and sort only the literal
    // runs between them.
    for segment in attrs.split_mut(|attr| attr.order_sensitive) {
        // Empty and singleton groups already have their only possible order.
        if segment.len() < 2 {
            continue;
        }
        if has_duplicate_static_names(segment) {
            continue;
        }
        sort_attribute_segment(tag_name, segment, options);
    }
}

/// Sort one contiguous group with no order-sensitive dynamic attributes.
fn sort_attribute_segment(tag_name: &str, attrs: &mut [ParsedAttribute], options: &FormatOptions) {
    match options.attribute_sort_order {
        AttributeSortOrder::Alphabetical => {
            // Compare borrowed names case-insensitively instead of caching
            // lowercased keys: segments now span directives, so a cached key
            // would allocate for most attributes of most elements. The order
            // is total (`original_index` is unique), so an unstable sort is
            // exact. Native elements may nominate a behavior-defining
            // attribute before the alphabetical tail of their ordinary
            // attributes. The tag-aware rank deliberately does not apply to
            // PascalCase Vue components.
            let merge_bind = options.merge_bind_and_non_bind_attrs;
            let ranked = has_native_attribute_semantic_ranks(tag_name);
            attrs.sort_unstable_by(|a, b| {
                // Most comparisons end at the priority group, so the name
                // keys are derived only when it ties.
                a.priority
                    .cmp(&b.priority)
                    .then_with(|| {
                        if !ranked {
                            return std::cmp::Ordering::Equal;
                        }
                        native_attribute_semantic_rank(tag_name, &a.name)
                            .cmp(&native_attribute_semantic_rank(tag_name, &b.name))
                    })
                    .then_with(|| {
                        let (a_group, a_base) = attr_sort_key(&a.name, merge_bind);
                        let (b_group, b_base) = attr_sort_key(&b.name, merge_bind);
                        a_group.cmp(&b_group).then_with(|| {
                            a_base
                                .bytes()
                                .map(|byte| byte.to_ascii_lowercase())
                                .cmp(b_base.bytes().map(|byte| byte.to_ascii_lowercase()))
                        })
                    })
                    .then(a.original_index.cmp(&b.original_index))
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

/// Rank behavior-defining attributes before the alphabetical tail for native
/// elements. Keep this table deliberately tag-aware: a prop with the same name
/// on a Vue component does not necessarily carry the HTML element's semantics.
fn native_attribute_semantic_rank(tag_name: &str, attr_name: &str) -> u8 {
    match (tag_name, attr_name) {
        ("button", name) if name.eq_ignore_ascii_case("type") => 0,
        _ => 1,
    }
}

/// Whether [`native_attribute_semantic_rank`] ranks any attribute of this tag.
fn has_native_attribute_semantic_ranks(tag_name: &str) -> bool {
    matches!(tag_name, "button")
}

/// A runtime binding, custom directive, dynamic argument, or mustache-backed
/// attribute can observe authored order or expand to runtime names, so each
/// bounds a sortable run.
pub(crate) fn is_order_sensitive_attribute(
    head: Option<&DirectiveName>,
    kind: Option<DirectiveKind>,
    value: Option<&str>,
) -> bool {
    let bounds_by_directive = match (head, kind) {
        (Some(head), Some(kind)) => match head.arg {
            Some(ArgSyntax::Dynamic(_)) => true,
            None => matches!(
                kind,
                DirectiveKind::Bind | DirectiveKind::On | DirectiveKind::Custom
            ),
            Some(ArgSyntax::Static(_)) => {
                matches!(kind, DirectiveKind::Bind | DirectiveKind::Custom)
            }
        },
        _ => false,
    };
    // The value is scanned only when the directive itself does not decide.
    bounds_by_directive || value.is_some_and(|value| value.contains("{{"))
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
fn attr_sort_key(name: &str, merge_bind: bool) -> (u8, &str) {
    // The base is compared ASCII-case-insensitively by the caller.
    let base = name
        .strip_prefix(':')
        .or_else(|| name.strip_prefix("v-bind:"));
    match base {
        // Strip bind prefix for comparison
        Some(base) if merge_bind => (0, base),
        // Non-bind first (0), then bind (1)
        Some(base) => (1, base),
        None => (0, name),
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
    let kind = head.as_ref().map(|head| DirectiveKind::of(head, name));
    attribute_priority_with_head(name, head.as_ref(), kind)
}

pub(crate) fn attribute_priority_with_head(
    name: &str,
    head: Option<&DirectiveName>,
    kind: Option<DirectiveKind>,
) -> u8 {
    if name == "is" {
        return 0;
    }
    if name == "id" {
        return 4;
    }
    if matches!(name, "ref" | "key" | "slot" | "slot-scope") {
        return 5;
    }
    let (Some(head), Some(kind)) = (head, kind) else {
        return 8;
    };
    match kind {
        // Patina currently keeps property shorthand in OtherAttrs.
        DirectiveKind::Bind if head.prefix == DirectivePrefix::Prop => 8,
        DirectiveKind::Bind => match static_directive_argument(name, head) {
            Some("is") => 0,
            Some("id") => 4,
            Some("ref" | "key" | "slot" | "slot-scope") => 5,
            _ => 8,
        },
        DirectiveKind::For => 1,
        DirectiveKind::If
        | DirectiveKind::ElseIf
        | DirectiveKind::Else
        | DirectiveKind::Show
        | DirectiveKind::Cloak => 2,
        DirectiveKind::Pre | DirectiveKind::Once => 3,
        DirectiveKind::Model => 6,
        DirectiveKind::Html | DirectiveKind::Text => 10,
        DirectiveKind::On => 9,
        DirectiveKind::Slot | DirectiveKind::Memo | DirectiveKind::Custom => 7,
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
