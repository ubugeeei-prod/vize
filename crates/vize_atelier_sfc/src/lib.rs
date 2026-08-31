//! Vue Single File Component (.vue) compiler.
//!
//! This module follows the Vue.js core structure for parsing and compilation:
//!
//! - `parse` - SFC parsing into descriptor blocks
//! - `compile_script` - Script/script setup compilation
//! - `compile_template` - Template block compilation (DOM and Vapor)
//! - `compile` - Main SFC compilation orchestration
//! - `style` - Style block compilation with scoped CSS
//! - `css` - Low-level CSS compilation with LightningCSS
//!
//! # Example
//!
//! ```ignore
//! use vize_atelier_sfc::{parse_sfc, compile_sfc, SfcParseOptions, SfcCompileOptions};
//!
//! let source = r#"
//! <script setup>
//! import { ref } from 'vue'
//! const count = ref(0)
//! </script>
//! <template>
//!   <button @click="count++">{{ count }}</button>
//! </template>
//! "#;
//!
//! let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
//! let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
//! println!("{}", result.code);
//! ```

#![allow(clippy::collapsible_match)]
#![allow(clippy::type_complexity)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::unnecessary_lazy_evaluations)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::only_used_in_recursion)]
#![cfg_attr(
    test,
    allow(unused_imports, clippy::disallowed_macros, clippy::disallowed_types)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods, clippy::needless_borrow))]

extern crate vize_s0 as vize_carton;

// Core modules - following Vue.js compiler-sfc structure
pub mod bundler;
pub mod compile;
pub mod compile_script;
pub mod compile_template;
pub mod croquis;
pub mod css;
pub mod module_shape;
pub mod rewrite_default;
pub mod script;
pub mod source_map;
pub mod style;
pub mod types;
pub mod vite_plugin;

/// Backward-compatible SFC parser module.
pub mod parse {
    use crate::types::{SfcDescriptor, SfcError, SfcParseOptions};

    /// Parse a Vue SFC into a zero-copy descriptor.
    #[inline]
    pub fn parse_sfc<'a>(
        source: &'a str,
        options: SfcParseOptions,
    ) -> Result<SfcDescriptor<'a>, SfcError> {
        vize_croquis::sfc::parse_sfc(source, options)
    }
}

// Re-exports for public API
pub use bundler::{
    BundlerCustomBlock, BundlerStyleBlock, SfcBlockAttribute, SfcSrcInfo, TemplateAssetTagRule,
    TemplateAssetUrl, collect_template_asset_urls, extract_custom_blocks, extract_src_info,
    extract_style_blocks, generate_bundler_scope_id, has_scoped_style, is_importable_asset_url,
    strip_css_comments_for_scoped, wrap_scoped_preprocessor_style,
};
#[allow(deprecated)]
pub use compile::compile_sfc_with_vue_parser_quirks;
pub use compile::{ScriptCompileResult, compile_sfc, compile_sfc_with_template_syntax};
pub use compile::{
    SfcScriptOutputMode, compile_sfc_for_adapter,
    compile_sfc_with_custom_elements_template_syntax_and_codegen_options,
    compile_sfc_with_template_syntax_and_codegen_options,
};
pub use compile_script::props::{
    script_setup_has_semantic_validator_candidates, validate_script_setup_semantics,
    validate_script_setup_semantics_located,
};
pub use css::{
    CssAstResult, CssCompileOptions, CssCompileResult, CssTargets, bundle_css, compile_css,
    compile_style_block, parse_css_ast, print_css_ast,
};
pub use parse::parse_sfc;
pub use script::{TypeResolutionBatchGuard, begin_type_resolution_batch};
pub use source_map::build_sfc_source_map;
pub use types::{
    BindingMetadata, BindingType, BlockLocation, PadOption, PropsDestructure, ScriptCompileOptions,
    SfcCompileOptions, SfcCompileResult, SfcCustomBlock, SfcDescriptor, SfcError, SfcMacroArtifact,
    SfcParseOptions, SfcScriptBlock, SfcStyleBlock, SfcTemplateBlock, StyleCompileOptions,
    TemplateCompileOptions,
};

// Re-export key types from dependencies
pub use vize_atelier_core::CompilerError;
pub use vize_atelier_dom::compile_template;

#[cfg(test)]
mod compile_tests;
#[cfg(test)]
mod parse_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod template_tests;
