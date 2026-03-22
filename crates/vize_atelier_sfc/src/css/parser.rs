//! Internal CSS compilation with LightningCSS.
//!
//! Handles parsing, minification, and printing of CSS using the LightningCSS engine.
//! This module is only available when the `native` feature is enabled.

use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::{ParserFlags, ParserOptions, StyleSheet};
use lightningcss::targets::Targets;
use vize_carton::{FxHashMap, String, ToCompactString};

/// Convert major version to LightningCSS format (major << 16)
pub(crate) fn version_to_u32(major: u32) -> u32 {
    major << 16
}

use super::CssModuleExport as VizeCssModuleExport;

/// CSS Modules compilation result
pub(crate) struct CssInternalResult {
    pub code: String,
    pub errors: Vec<String>,
    pub exports: Option<FxHashMap<String, VizeCssModuleExport>>,
}

/// Internal CSS compilation with owned strings to avoid borrow issues
pub(crate) fn compile_css_internal(
    css: &str,
    filename: &str,
    minify: bool,
    targets: Targets,
    custom_media: bool,
    css_modules: bool,
) -> CssInternalResult {
    let mut flags = ParserFlags::NESTING | ParserFlags::DEEP_SELECTOR_COMBINATOR;
    if custom_media {
        flags |= ParserFlags::CUSTOM_MEDIA;
    }

    let css_modules_config = if css_modules {
        Some(lightningcss::css_modules::Config {
            pattern: lightningcss::css_modules::Pattern::default(),
            ..Default::default()
        })
    } else {
        None
    };

    let parser_options = ParserOptions {
        filename: filename.into(),
        flags,
        css_modules: css_modules_config,
        ..Default::default()
    };

    let mut stylesheet = match StyleSheet::parse(css, parser_options) {
        Ok(ss) => ss,
        Err(e) => {
            let mut errors = Vec::with_capacity(1);
            let mut message = String::from("CSS parse error: ");
            message.push_str(&e.to_compact_string());
            errors.push(message);
            return CssInternalResult {
                code: css.to_compact_string(),
                errors,
                exports: None,
            };
        }
    };

    if minify {
        if let Err(e) = stylesheet.minify(lightningcss::stylesheet::MinifyOptions {
            targets,
            ..Default::default()
        }) {
            let mut errors = Vec::with_capacity(1);
            let mut message = String::from("CSS minify error: ");
            use std::fmt::Write as _;
            let _ = write!(&mut message, "{:?}", e);
            errors.push(message);
            return CssInternalResult {
                code: css.to_compact_string(),
                errors,
                exports: None,
            };
        }
    }

    let printer_options = PrinterOptions {
        minify,
        targets,
        ..Default::default()
    };

    match stylesheet.to_css(printer_options) {
        Ok(result) => {
            let exports = result.exports.map(|export_map| {
                export_map
                    .into_iter()
                    .map(|(original, export)| {
                        (
                            original.to_compact_string(),
                            VizeCssModuleExport {
                                name: export.name.to_compact_string(),
                                is_referenced: export.is_referenced,
                            },
                        )
                    })
                    .collect()
            });

            CssInternalResult {
                code: result.code.into(),
                errors: vec![],
                exports,
            }
        }
        Err(e) => {
            let mut errors = Vec::with_capacity(1);
            let mut message = String::from("CSS print error: ");
            use std::fmt::Write as _;
            let _ = write!(&mut message, "{:?}", e);
            errors.push(message);
            CssInternalResult {
                code: css.to_compact_string(),
                errors,
                exports: None,
            }
        }
    }
}
