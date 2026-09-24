//! # vize_glyph
//!
//! Glyph - The beautiful letterforms for Vize.
//! High-performance formatter for Vue.js Single File Components.
//!
//! ## Name Origin
//!
//! **Glyph** (/ɡlɪf/) refers to the visual representation of a character -
//! the elegant form that gives meaning to written symbols. In typography and
//! calligraphy, glyphs are carefully crafted to be both beautiful and legible.
//! `vize_glyph` shapes Vue SFC code into its most readable and consistent form.
//!
//! ## Performance
//!
//! This crate is designed for maximum performance:
//! - Arena allocation via `vize_s0::Allocator` for minimal heap allocations
//! - Zero-copy parsing where possible
//! - SIMD-accelerated string operations via `memchr`
//! - Efficient buffer management with pre-allocated capacity
//!
//! ## Example
//!
//! ```ignore
//! use vize_glyph::{format_sfc, FormatOptions};
//!
//! let source = r#"
//! <script setup>
//! import {ref} from 'vue'
//! const count=ref(0)
//! </script>
//! <template>
//!   <button @click="count++">{{count}}</button>
//! </template>
//! "#;
//! let options = FormatOptions::default();
//! let result = format_sfc(source, &options).unwrap();
//! println!("{}", result.code);
//! ```

#![cfg_attr(
    test,
    expect(
        clippy::disallowed_macros,
        clippy::disallowed_types,
        reason = "unit tests build fixtures with format! and std strings"
    )
)]

mod attribute;
mod error;
mod formatter;
mod json;
mod options;
mod script;
mod style;
mod template;

pub use error::*;
pub use formatter::*;
pub use json::{format_json_source as format_json, format_jsonc_source as format_jsonc};
pub use options::*;
pub use vize_s0::Allocator;
use vize_s0::String;

/// Format a Vue SFC source string
///
/// This is the main entry point for formatting Vue Single File Components.
/// Uses arena allocation for efficient memory management.
#[inline]
pub fn format_sfc(source: &str, options: &FormatOptions) -> Result<FormatResult, FormatError> {
    let allocator = Allocator::with_capacity(source.len() * 2);
    format_sfc_with_allocator(source, options, &allocator)
}

/// Format a Vue SFC source string with a provided allocator
///
/// Use this when you want to reuse an allocator across multiple format operations.
#[inline]
pub fn format_sfc_with_allocator(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
) -> Result<FormatResult, FormatError> {
    let formatter = GlyphFormatter::new(options, allocator);
    formatter.format(source)
}

/// Format only the script/TypeScript content
#[inline]
pub fn format_script(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let allocator = Allocator::with_capacity(source.len() * 2);
    script::format_ts_script_content_stable(source, options, &allocator)
}

/// Format only script content with an explicit JavaScript/TypeScript source type.
#[inline]
pub fn format_script_with_source_type(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    source_type: oxc_span::SourceType,
) -> Result<String, FormatError> {
    script::format_script_content_stable(source, options, allocator, source_type)
}

/// Format only the template content
#[inline]
pub fn format_template(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    template::format_template_content(source, options)
}

/// Format only the CSS/style content
#[inline]
pub fn format_style(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    style::format_style_content(source, options)
}

#[cfg(test)]
mod tests;
