//! NAPI bindings for Vue SFC formatting.

#![allow(clippy::disallowed_types)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use vize_glyph::{format_sfc_with_allocator, Allocator, FormatOptions};

/// Format options for NAPI.
#[napi(object)]
#[derive(Default)]
pub struct FormatOptionsNapi {
    pub print_width: Option<u32>,
    pub tab_width: Option<u8>,
    pub use_tabs: Option<bool>,
    pub semi: Option<bool>,
    pub single_quote: Option<bool>,
    pub sort_attributes: Option<bool>,
    pub single_attribute_per_line: Option<bool>,
    pub max_attributes_per_line: Option<u32>,
    pub normalize_directive_shorthands: Option<bool>,
}

/// Format result for NAPI.
#[napi(object)]
pub struct FormatResultNapi {
    pub code: String,
    pub changed: bool,
}

fn apply_options(opts: FormatOptionsNapi) -> FormatOptions {
    let mut options = FormatOptions::default();

    if let Some(value) = opts.print_width {
        options.print_width = value;
    }
    if let Some(value) = opts.tab_width {
        options.tab_width = value;
    }
    if let Some(value) = opts.use_tabs {
        options.use_tabs = value;
    }
    if let Some(value) = opts.semi {
        options.semi = value;
    }
    if let Some(value) = opts.single_quote {
        options.single_quote = value;
    }
    if let Some(value) = opts.sort_attributes {
        options.sort_attributes = value;
    }
    if let Some(value) = opts.single_attribute_per_line {
        options.single_attribute_per_line = value;
    }
    if let Some(value) = opts.max_attributes_per_line {
        options.max_attributes_per_line = Some(value);
    }
    if let Some(value) = opts.normalize_directive_shorthands {
        options.normalize_directive_shorthands = value;
    }

    options
}

/// Format a Vue SFC source string.
#[napi(js_name = "formatSfc")]
pub fn format_sfc_napi(
    source: String,
    options: Option<FormatOptionsNapi>,
) -> Result<FormatResultNapi> {
    let options = apply_options(options.unwrap_or_default());
    let allocator = Allocator::with_capacity(source.len() * 2);
    let result = format_sfc_with_allocator(&source, &options, &allocator)
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;

    Ok(FormatResultNapi {
        code: result.code.into(),
        changed: result.changed,
    })
}
