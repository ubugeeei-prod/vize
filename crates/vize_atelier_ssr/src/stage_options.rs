//! Per-stage option construction for SSR template compilation.
//!
//! Keeps the parse/transform option wiring out of `lib.rs` so the pipeline
//! entry points stay focused on flow.

use vize_atelier_core::options::{ParserOptions, TransformOptions};

use crate::compile::get_namespace;
use crate::options::SsrCompilerOptions;

/// Parser options for the SSR pipeline.
pub(crate) fn parser_options(options: &SsrCompilerOptions) -> ParserOptions {
    ParserOptions {
        is_void_tag: vize_s0::is_void_tag,
        is_native_tag: Some(vize_s0::is_native_tag),
        custom_renderer: options.custom_renderer,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        comments: options.comments,
        experimental_in_tag_comments: options.experimental_in_tag_comments,
        dialect: options.dialect,
        ..ParserOptions::default()
    }
}

/// Transform options for the SSR pipeline.
///
/// SSR always uses prefix identifiers and disables hoisting/caching.
pub(crate) fn transform_options(options: &SsrCompilerOptions) -> TransformOptions {
    TransformOptions {
        prefix_identifiers: true, // SSR always uses prefix
        hoist_static: false,      // No hoisting in SSR
        cache_handlers: false,    // No caching in SSR
        scope_id: options.scope_id.clone(),
        ssr: true,
        is_ts: options.is_ts,
        inline: options.inline,
        custom_renderer: options.custom_renderer,
        experimental_patterned_template: options.experimental_patterned_template,
        binding_metadata: options.binding_metadata.clone(),
        dialect: options.dialect,
        ..Default::default()
    }
}
