//! Directive shorthand and sort-priority normalization.

use super::super::attributes::attribute_priority_with_head;
use super::{
    custom_attribute_priority, directive_head, format_directive_value,
    should_format_expression_with_head,
};
use crate::options::FormatOptions;
use vize_l0::{String, ToCompactString, cstr};
use vize_l1::dialect::vue3::DirectiveKind;
use vize_l1::markup::DirectivePrefix;

pub(crate) fn normalize_attribute_with_vue_version(
    name: &str,
    value: Option<String>,
    options: &FormatOptions,
    vue_version: crate::VueVersion,
    attribute_depth: usize,
) -> (String, Option<String>, usize, bool, bool) {
    // Normalize directive shorthands (only if enabled)
    let head = directive_head(name);
    let normalized_shorthand: Option<String> = head
        .as_ref()
        .filter(|head| {
            options.normalize_directive_shorthands && head.prefix == DirectivePrefix::Full
        })
        .and_then(|head| {
            let sigil = match DirectiveKind::of(head, name) {
                DirectiveKind::Bind => ':',
                DirectiveKind::On => '@',
                DirectiveKind::Slot => '#',
                _ => return None,
            };
            let rest = name.get(head.name.end as usize..)?.strip_prefix(':')?;
            Some(cstr!("{sigil}{rest}"))
        });
    let (normalized_name, normalized_head) = match normalized_shorthand {
        Some(normalized_name) => {
            let normalized_head = directive_head(&normalized_name);
            (normalized_name, normalized_head)
        }
        None => (name.to_compact_string(), head),
    };

    // Format JS expressions in directive values
    let mut indent_multiline_value = false;
    let mut owns_value_lines = false;
    let formatted_value = value.map(|v| {
        if should_format_expression_with_head(&normalized_name, normalized_head.as_ref()) {
            let (formatted, should_indent, owns_lines) =
                format_directive_value(&normalized_name, &v, options, vue_version, attribute_depth);
            indent_multiline_value = should_indent;
            owns_value_lines = owns_lines;
            formatted
        } else {
            v
        }
    });

    let priority = if let Some(ref groups) = options.attribute_groups {
        custom_attribute_priority(&normalized_name, groups)
    } else {
        usize::from(attribute_priority_with_head(
            &normalized_name,
            normalized_head.as_ref(),
        ))
    };

    (
        normalized_name,
        formatted_value,
        priority,
        indent_multiline_value,
        owns_value_lines,
    )
}
