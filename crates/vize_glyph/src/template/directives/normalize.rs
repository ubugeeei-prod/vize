//! Directive shorthand and sort-priority normalization.

use super::super::attributes::attribute_priority;
use super::{custom_attribute_priority, format_directive_value, should_format_expression};
use crate::options::FormatOptions;
use vize_l0::{String, ToCompactString, cstr};

/// Normalize directive shorthands and assign sort priority.
pub(crate) fn normalize_attribute(
    name: &str,
    value: Option<String>,
    options: &FormatOptions,
) -> (String, Option<String>, u8, bool) {
    // Normalize directive shorthands (only if enabled)
    let normalized_name: String = if options.normalize_directive_shorthands {
        if let Some(rest) = name.strip_prefix("v-bind:") {
            cstr!(":{rest}")
        } else if let Some(rest) = name.strip_prefix("v-on:") {
            cstr!("@{rest}")
        } else if let Some(rest) = name.strip_prefix("v-slot:") {
            cstr!("#{rest}")
        } else {
            name.to_compact_string()
        }
    } else {
        name.to_compact_string()
    };

    // Format JS expressions in directive values
    let mut indent_multiline_value = false;
    let formatted_value = value.map(|v| {
        if should_format_expression(&normalized_name) {
            let (formatted, should_indent) = format_directive_value(&normalized_name, &v, options);
            indent_multiline_value = should_indent;
            formatted
        } else {
            v
        }
    });

    let priority = if let Some(ref groups) = options.attribute_groups {
        custom_attribute_priority(&normalized_name, groups)
    } else {
        attribute_priority(&normalized_name)
    };

    (
        normalized_name,
        formatted_value,
        priority,
        indent_multiline_value,
    )
}
