//! CSS compilation using LightningCSS.
//!
//! Provides high-performance CSS parsing, transformation, and minification.
//! When the `native` feature is disabled (e.g., for wasm builds), a simple
//! passthrough implementation is used.
//!
//! This module is organized into:
//! - Types and public API (this file)
//! - `parser`: internal CSS compilation with LightningCSS
//! - `transform`: v-bind() extraction and byte-level utilities
//! - `scoped`: scoped CSS transformation (:deep, :slotted, :global)

#[cfg(not(feature = "native"))]
use vize_carton::ToCompactString;
use vize_carton::{FxHashMap, String};
#[cfg(feature = "native")]
mod parser;
mod scoped;
#[cfg(test)]
mod tests;
mod transform;

use serde::{Deserialize, Serialize};
use vize_carton::Bump;

use crate::types::SfcStyleBlock;

use self::scoped::apply_scoped_css;
use self::transform::extract_and_transform_v_bind;

/// CSS compilation options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssCompileOptions {
    /// Scope ID for scoped CSS (e.g., "data-v-abc123")
    #[serde(default)]
    pub scope_id: Option<String>,

    /// Whether to apply scoped CSS transformation
    #[serde(default)]
    pub scoped: bool,

    /// Whether to minify the output
    #[serde(default)]
    pub minify: bool,

    /// Whether to generate source maps
    #[serde(default)]
    pub source_map: bool,

    /// Browser targets for autoprefixing
    #[serde(default)]
    pub targets: Option<CssTargets>,

    /// Filename for error reporting
    #[serde(default)]
    pub filename: Option<String>,

    /// Whether to enable custom media query resolution
    #[serde(default)]
    pub custom_media: bool,

    /// Enable CSS Modules — scopes class names, IDs, and keyframes.
    /// When enabled, the result includes an `exports` map of original → hashed names.
    #[serde(default)]
    pub css_modules: bool,
}

/// Browser targets for CSS autoprefixing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssTargets {
    #[serde(default)]
    pub chrome: Option<u32>,
    #[serde(default)]
    pub firefox: Option<u32>,
    #[serde(default)]
    pub safari: Option<u32>,
    #[serde(default)]
    pub edge: Option<u32>,
    #[serde(default)]
    pub ios: Option<u32>,
    #[serde(default)]
    pub android: Option<u32>,
}

/// A single CSS Modules export entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModuleExport {
    /// The compiled (hashed) name
    pub name: String,
    /// Whether this export is actually referenced in the CSS
    pub is_referenced: bool,
}

/// CSS compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssCompileResult {
    /// Compiled CSS code
    pub code: String,

    /// Source map (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<String>,

    /// CSS variables found (from v-bind())
    #[serde(default)]
    pub css_vars: Vec<String>,

    /// Errors during compilation
    #[serde(default)]
    pub errors: Vec<String>,

    /// Warnings during compilation
    #[serde(default)]
    pub warnings: Vec<String>,

    /// CSS Modules exports — original name → compiled name.
    /// Only populated when `css_modules: true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exports: Option<FxHashMap<String, CssModuleExport>>,
}

/// Compile CSS using LightningCSS (native feature enabled)
#[cfg(feature = "native")]
pub fn compile_css(css: &str, options: &CssCompileOptions) -> CssCompileResult {
    let bump = Bump::new();
    let filename = options.filename.as_deref().unwrap_or("style.css");

    // Extract v-bind() expressions before parsing
    let (processed_css, css_vars) = extract_and_transform_v_bind(&bump, css);

    // Apply scoped transformation if needed
    let scoped_css = if options.scoped {
        if let Some(ref scope_id) = options.scope_id {
            apply_scoped_css(&bump, processed_css, scope_id)
        } else {
            processed_css
        }
    } else {
        processed_css
    };

    // Apply targets for autoprefixing
    let targets = options
        .targets
        .as_ref()
        .map(|t| t.to_lightningcss_targets())
        .unwrap_or_default();

    // Parse and process CSS
    let result = parser::compile_css_internal(
        scoped_css,
        filename,
        options.minify,
        targets,
        options.custom_media,
        options.css_modules,
    );

    CssCompileResult {
        code: result.code,
        map: None,
        css_vars,
        errors: result.errors,
        warnings: vec![],
        exports: result.exports,
    }
}

/// Compile CSS (wasm fallback - no LightningCSS)
#[cfg(not(feature = "native"))]
pub fn compile_css(css: &str, options: &CssCompileOptions) -> CssCompileResult {
    let bump = Bump::new();

    // Extract v-bind() expressions before parsing
    let (processed_css, css_vars) = extract_and_transform_v_bind(&bump, css);

    // Apply scoped transformation if needed
    let scoped_css = if options.scoped {
        if let Some(ref scope_id) = options.scope_id {
            apply_scoped_css(&bump, processed_css, scope_id)
        } else {
            processed_css
        }
    } else {
        processed_css
    };

    CssCompileResult {
        code: scoped_css.to_compact_string(),
        map: None,
        css_vars,
        errors: vec![],
        warnings: vec![],
        exports: None,
    }
}

/// Compile a style block
pub fn compile_style_block(style: &SfcStyleBlock, options: &CssCompileOptions) -> CssCompileResult {
    let mut opts = options.clone();
    opts.scoped = style.scoped || opts.scoped;
    compile_css(&style.content, &opts)
}

#[cfg(feature = "native")]
impl CssTargets {
    pub(crate) fn to_lightningcss_targets(&self) -> lightningcss::targets::Targets {
        let mut browsers = lightningcss::targets::Browsers::default();

        if let Some(v) = self.chrome {
            browsers.chrome = Some(parser::version_to_u32(v));
        }
        if let Some(v) = self.firefox {
            browsers.firefox = Some(parser::version_to_u32(v));
        }
        if let Some(v) = self.safari {
            browsers.safari = Some(parser::version_to_u32(v));
        }
        if let Some(v) = self.edge {
            browsers.edge = Some(parser::version_to_u32(v));
        }
        if let Some(v) = self.ios {
            browsers.ios_saf = Some(parser::version_to_u32(v));
        }
        if let Some(v) = self.android {
            browsers.android = Some(parser::version_to_u32(v));
        }

        lightningcss::targets::Targets::from(browsers)
    }
}
