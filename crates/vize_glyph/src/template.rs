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
use vize_s0::String;

use formatter::TemplateFormatter;
use helpers::is_whitespace;

/// Format Vue template content.
#[inline]
pub fn format_template_content(
    source: &str,
    options: &FormatOptions,
) -> Result<String, FormatError> {
    let bytes = source.as_bytes();

    // Fast path: all whitespace
    if bytes.iter().all(|&b| is_whitespace(b)) {
        return Ok(String::default());
    }

    let formatter = TemplateFormatter::new(options);
    formatter.format(bytes)
}

#[cfg(test)]
mod tests;
