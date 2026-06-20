//! Template and Art completion providers.
//!
//! Handles completions for template directives, built-in components,
//! Art blocks, and variant blocks.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod art;
mod bindings;
mod component_meta;
mod components;
mod directives;
mod self_component;
mod tag_context;

use tower_lsp::lsp_types::CompletionItem;

use super::is_inside_html_comment;
use crate::ide::IdeContext;

use bindings::analyzed_template_binding_completions;
use components::builtin_component_completions;

pub(crate) use art::{complete_art, complete_inline_art};
pub(crate) use component_meta::CachedComponentMetadata;
pub(crate) use directives::{contextual_directive_completions, vize_directive_completions};
// Consumed via `template::*` by unit tests in the parent completion module; the
// `allow` keeps non-test builds warning-free while preserving the public path.
#[cfg_attr(not(test), allow(unused_imports))]
pub(crate) use directives::{directive_completions, petite_vue_directive_completions};

/// Get completions for template context.
pub(crate) fn complete_template(ctx: &IdeContext) -> Vec<CompletionItem> {
    // If cursor is inside an HTML comment, offer @vize: directive completions only
    if is_inside_html_comment(&ctx.content, ctx.offset) {
        return vize_directive_completions();
    }

    // `$style.|` in a template expression should resolve to class names
    // declared in the SFC's `<style module>` blocks.
    if let Some(items) = bindings::css_module_class_completions(ctx) {
        return items;
    }

    let mut items_vec = Vec::new();

    // Add Vue directives
    items_vec.extend(contextual_directive_completions(ctx));

    // Add built-in components
    items_vec.extend(builtin_component_completions());
    if ctx.state.lsp_features().legacy_vue2 {
        items_vec.extend(components::legacy_vue2_component_completions());
    }
    items_vec.extend(component_meta::component_surface_completions(ctx));

    if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
        items_vec.extend(bindings::template_snippets());
        return items_vec;
    }

    items_vec.extend(analyzed_template_binding_completions(ctx, true));

    // Add common template snippets
    items_vec.extend(bindings::template_snippets());

    items_vec
}

/// Corsa-path supplement for Vue 2.7 / Nuxt 2 constructs that the TypeScript
/// virtual document cannot express: Nuxt 2 built-in components (`<nuxt-link>`,
/// `<client-only>`, …) and the legacy-filtered Options API bindings. This stays
/// gated on `legacy_vue2` deliberately — it has no Vue 3 Options API counterpart
/// because Options API `data`/`computed`/`methods`/`props` ARE emitted into the
/// virtual TS as accessible bindings (see `generate_options_api_variables`), so
/// Corsa already surfaces them; the synchronous fallback covers them via
/// [`analyzed_template_binding_completions`]. Options API is standard Vue 3 and
/// is treated like the default path here, not like legacy.
pub(crate) fn legacy_vue2_template_completions(ctx: &IdeContext) -> Vec<CompletionItem> {
    if !ctx.state.lsp_features().legacy_vue2 {
        return Vec::new();
    }

    if crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
        analyzed_template_binding_completions(ctx, false)
    } else {
        components::legacy_vue2_component_completions()
    }
}
