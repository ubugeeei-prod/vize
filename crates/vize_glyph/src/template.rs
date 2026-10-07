//! High-performance template formatting for Vue SFC.
//!
//! Features:
//! - Proper indentation and nesting
//! - Directive shorthand normalization (`v-bind:` -> `:`, `v-on:` -> `@`, `v-slot:` -> `#`)
//! - Interpolation spacing normalization (`{{expr}}` -> `{{ expr }}`)
//! - JS expression formatting in directive values via oxc_formatter
//! - Attribute sorting following Vue style guide order
//! - `single_attribute_per_line` support with `bracket_same_line`

mod attributes;
mod directives;
mod formatter;
pub(crate) mod helpers;
pub(crate) mod literal_lines;
mod vue_filters;

/// Native HTML elements whose authored text is whitespace-significant.
///
/// Keep this list shared with the SFC indentation mask: if the two formatter
/// layers disagree, each pass can add another indentation level. Vue treats
/// names containing uppercase ASCII as components, so callers must match
/// these lowercase names exactly.
pub(crate) const WHITESPACE_SIGNIFICANT_NATIVE_ELEMENTS: [&str; 3] = ["pre", "textarea", "listing"];

#[cfg(test)]
mod attribute_priority_tests;

use crate::{error::FormatError, options::FormatOptions};
use vize_l0::String;

use formatter::TemplateFormatter;
use helpers::is_whitespace;

/// Format Vue template content.
#[inline]
pub fn format_template_content(
    source: &str,
    options: &FormatOptions,
) -> Result<String, FormatError> {
    format_template_content_with_vue_version(source, options, crate::VueVersion::V3)
}

pub(crate) fn format_template_content_with_vue_version(
    source: &str,
    options: &FormatOptions,
    vue_version: crate::VueVersion,
) -> Result<String, FormatError> {
    format_template_content_with_base_depth(source, options, vue_version, 0)
}

pub(crate) fn format_template_content_with_base_depth(
    source: &str,
    options: &FormatOptions,
    vue_version: crate::VueVersion,
    base_depth: usize,
) -> Result<String, FormatError> {
    if options.end_of_line == crate::EndOfLine::Auto {
        return options.format_with_source_line_ending(source, |options| {
            format_template_content_with_base_depth(source, options, vue_version, base_depth)
        });
    }
    let bytes = source.as_bytes();

    // Fast path: all whitespace
    if bytes.iter().all(|&b| is_whitespace(b)) {
        return Ok(String::default());
    }

    let formatter = TemplateFormatter::new(options, vue_version, base_depth);
    formatter.format(bytes)
}

#[cfg(test)]
mod tests;
