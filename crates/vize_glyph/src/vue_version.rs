//! Explicit Vue version selection without changing serialized formatting options.

use crate::{Allocator, FormatError, FormatOptions, FormatResult, GlyphFormatter};
use vize_l0::String;

pub use vize_l0::config::VueVersion;

/// Format an SFC using the project's explicit Vue version.
pub fn format_sfc_with_vue_version(
    source: &str,
    options: &FormatOptions,
    vue_version: VueVersion,
) -> Result<FormatResult, FormatError> {
    let allocator = Allocator::with_capacity(source.len() * 2);
    format_sfc_with_allocator_and_vue_version(source, options, &allocator, vue_version)
}

/// Format an SFC with a reusable allocator and an explicit Vue version.
pub fn format_sfc_with_allocator_and_vue_version(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    vue_version: VueVersion,
) -> Result<FormatResult, FormatError> {
    GlyphFormatter::new_with_vue_version(options, allocator, vue_version).format(source)
}

/// Format template content using the project's explicit Vue version.
pub fn format_template_with_vue_version(
    source: &str,
    options: &FormatOptions,
    vue_version: VueVersion,
) -> Result<String, FormatError> {
    crate::template::format_template_content_with_vue_version(source, options, vue_version)
}
