//! Per-stage option construction for DOM template compilation.
//!
//! Keeps the parse/transform option wiring out of `compile.rs` so that entry
//! point stays focused on pipeline flow.

use vize_atelier_core::options::{ParserOptions, TransformOptions};

use crate::namespace::get_namespace;
use crate::options::DomCompilerOptions;

/// Parser options with DOM-specific settings.
pub(super) fn parser_options(options: &DomCompilerOptions) -> ParserOptions {
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

/// Transform options for the DOM-specific transform steps.
///
/// `BindingMetadata` is passed directly (no string conversion needed).
pub(super) fn transform_options(options: &DomCompilerOptions) -> TransformOptions {
    TransformOptions {
        prefix_identifiers: options.prefix_identifiers,
        hoist_static: options.hoist_static,
        cache_handlers: options.cache_handlers,
        scope_id: options.scope_id.clone(),
        ssr: options.ssr,
        is_ts: options.is_ts,
        inline: options.inline,
        custom_renderer: options.custom_renderer,
        experimental_patterned_template: options.experimental_patterned_template,
        binding_metadata: options.binding_metadata.clone(),
        dialect: options.dialect,
        ..Default::default()
    }
}
