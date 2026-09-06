use crate::{
    attribute::write_attr_value,
    options::{AttributeSortOrder, FormatOptions},
};
use vize_s0::String;

use super::helpers::template_literal_state_after_line_from;

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
}

/// Sort attributes based on the configured options.
pub(crate) fn sort_attributes(attrs: &mut [ParsedAttribute], options: &FormatOptions) {
    for segment in attrs.split_mut(is_ordering_barrier) {
        if has_duplicate_static_names(segment) {
            continue;
        }
        sort_attribute_segment(segment, options);
    }
}

/// Sort one contiguous group that cannot cross an object directive spread.
///
/// Vue applies object `v-bind` and `v-on` directives in source order. Moving
/// an attribute across either directive changes the order of `mergeProps`
/// arguments and can therefore change which value wins at runtime.
fn sort_attribute_segment(attrs: &mut [ParsedAttribute], options: &FormatOptions) {
    match options.attribute_sort_order {
        AttributeSortOrder::Alphabetical => {
            // Decorate-sort-undecorate: `attr_sort_key` lowercases the name,
            // so computing it inside a comparator re-allocates O(n log n)
            // times. `sort_by_cached_key` evaluates the key closure exactly
            // once per attribute and caches it, then sorts on the cached
            // tuples. Static attrs still sort alphabetically inside their
            // group, while dynamic attrs preserve authored order inside their
            // lint group so formatter output does not reshuffle listener or
            // binding evaluation within an otherwise valid category.
            let merge_bind = options.merge_bind_and_non_bind_attrs;
            attrs.sort_by_cached_key(|attr| {
                let is_dynamic = is_dynamic_attribute_name(&attr.name);
                let keep_authored_order =
                    is_dynamic && !(merge_bind && is_bind_attribute_name(&attr.name));
                let (group, base) = if keep_authored_order {
                    (1, String::new(""))
                } else {
                    attr_sort_key(&attr.name, merge_bind)
                };
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

fn is_ordering_barrier(attr: &ParsedAttribute) -> bool {
    let name = attr.name.as_str();
    is_object_spread_directive(name)
        || attr
            .value
            .as_deref()
            .is_some_and(|value| value.contains("{{"))
}

fn is_object_spread_directive(name: &str) -> bool {
    matches!(name, ":" | "@" | "v-bind" | "v-on")
        || name.starts_with("v-bind.")
        || name.starts_with("v-on.")
}

fn is_dynamic_attribute_name(name: &str) -> bool {
    name.starts_with([':', '@', '#', '.']) || name.starts_with("v-")
}

fn is_bind_attribute_name(name: &str) -> bool {
    name.starts_with([':', '.']) || name.starts_with("v-bind:")
}

fn has_duplicate_static_names(attrs: &[ParsedAttribute]) -> bool {
    attrs.iter().enumerate().any(|(index, attr)| {
        attrs[..index]
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
pub(crate) fn attribute_priority(name: &str) -> u8 {
    // Exact directive name or an `:arg`/`.mod` boundary (so `v-models` etc. fall through to 7).
    fn matches_directive(name: &str, directive: &str) -> bool {
        name.strip_prefix(directive)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with([':', '.']))
    }
    if matches!(name, "is" | ":is" | "v-bind:is") {
        return 0;
    }
    if name == "v-for" {
        return 1;
    }
    if matches!(name, "v-if" | "v-else-if" | "v-else" | "v-show" | "v-cloak") {
        return 2;
    }
    if matches!(name, "v-pre" | "v-once") {
        return 3;
    }
    if matches!(name, "id" | ":id" | "v-bind:id") {
        return 4;
    }
    if matches!(
        name,
        "ref"
            | "key"
            | "slot"
            | "slot-scope"
            | ":ref"
            | ":key"
            | ":slot"
            | ":slot-scope"
            | "v-bind:ref"
            | "v-bind:key"
            | "v-bind:slot"
            | "v-bind:slot-scope"
    ) {
        return 5;
    }
    if matches_directive(name, "v-model") {
        return 6;
    }
    if matches!(name, "v-html" | "v-text") {
        return 10;
    }
    // Events precede the directive fallback so `v-on` cannot swallow `v-once`.
    if name.starts_with('@') || matches_directive(name, "v-on") {
        return 9;
    }
    // Slots and every other directive form are patina's OtherDirectives.
    if name.starts_with('#') || name.starts_with("v-slot") {
        return 7;
    }
    // Bindings stay with plain attributes (patina: OtherAttrs).
    if name.starts_with(':') || matches_directive(name, "v-bind") || name.starts_with('.') {
        return 8;
    }
    if name.starts_with("v-") {
        return 7;
    }
    // Plain attributes (class, style, data-*, ...).
    8
}

/// Render an attribute back to its string representation.
pub(crate) fn render_attribute(attr: &ParsedAttribute) -> String {
    match &attr.value {
        Some(value) => {
            let mut rendered = String::with_capacity(attr.name.len() + value.len() + 3);
            rendered.push_str(&attr.name);
            rendered.push('=');
            write_attr_value(value, |segment| rendered.push_str(segment));
            rendered
        }
        None => attr.name.clone(),
    }
}

pub(crate) fn rendered_attribute_is_multiline(attr: &str) -> bool {
    attr.contains('\n')
}

pub(crate) fn should_use_multiline_attrs(
    options: &FormatOptions,
    tag_name: &str,
    attrs: &[ParsedAttribute],
    rendered: &[String],
    depth: usize,
    indent: &[u8],
) -> bool {
    if rendered
        .iter()
        .any(|rendered| rendered_attribute_is_multiline(rendered))
    {
        return true;
    }

    if attrs.len() <= 1 {
        return false;
    }

    if options.single_attribute_per_line {
        return true;
    }

    if let Some(max) = options.max_attributes_per_line {
        return attrs.len() > max as usize;
    }

    let indent_len = indent.len() * depth;
    let tag_len = 1 + tag_name.len();
    let attrs_len: usize = rendered.iter().map(|a| 1 + a.len()).sum();
    let closing_len = 1;

    indent_len + tag_len + attrs_len + closing_len > options.print_width as usize
}

pub(crate) fn write_rendered_attributes(
    output: &mut Vec<u8>,
    attrs: &[ParsedAttribute],
    rendered: &[String],
    newline: &[u8],
    indent: &[u8],
    depth: usize,
    max_per_line: usize,
) {
    debug_assert_eq!(attrs.len(), rendered.len());
    let mut line_count = 0;
    for (attr, rendered) in attrs.iter().zip(rendered) {
        let attr_is_multiline = rendered_attribute_is_multiline(rendered);
        if line_count == 0 || attr_is_multiline {
            output.extend_from_slice(newline);
            write_indent(output, indent, depth);
        } else {
            output.push(b' ');
        }
        write_rendered_attribute(
            output,
            rendered,
            newline,
            indent,
            depth,
            attr.indent_multiline_value,
        );
        if attr_is_multiline || line_count + 1 >= max_per_line {
            line_count = 0;
        } else {
            line_count += 1;
        }
    }
}

fn write_rendered_attribute(
    output: &mut Vec<u8>,
    attr: &str,
    newline: &[u8],
    indent: &[u8],
    continuation_depth: usize,
    indent_continuation: bool,
) {
    let mut lines = attr.split('\n');
    let mut in_template_literal = false;
    if let Some(first) = lines.next() {
        let first = first.trim_end_matches('\r');
        output.extend_from_slice(first.as_bytes());
        in_template_literal = template_literal_state_after_line_from(false, first);
    }

    for line in lines {
        output.extend_from_slice(newline);
        let line = line.trim_end_matches('\r');
        // Every byte between the backticks is part of the string's runtime
        // value, so a line that *starts* inside a template literal is emitted
        // exactly as the expression formatter produced it — no attribute
        // indent in front of it, no leading whitespace stripped off it.
        //
        // Rewriting that whitespace was not only a rendered-output change: it
        // also moved the column at which every embedded `${…}` starts. The
        // expression formatter measures its line-break budget from that
        // column, so the next `vize fmt` pass — reading back the re-indented
        // literal — made a different wrap decision and produced a different
        // file. (#3379)
        if indent_continuation && !in_template_literal {
            write_indent(output, indent, continuation_depth);
        }
        output.extend_from_slice(line.as_bytes());
        in_template_literal = template_literal_state_after_line_from(in_template_literal, line);
    }
}

fn write_indent(output: &mut Vec<u8>, indent: &[u8], depth: usize) {
    for _ in 0..depth {
        output.extend_from_slice(indent);
    }
}

#[cfg(test)]
mod tests;
