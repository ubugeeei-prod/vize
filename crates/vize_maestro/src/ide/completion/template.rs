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
mod component_cache;
mod component_docs;
mod component_meta;
mod components;
mod directives;
mod native;
#[cfg(test)]
mod native_tests;
mod self_component;
mod slot_outlets;
mod tag_context;

use tower_lsp::lsp_types::{CompletionItem, CompletionTextEdit, Position, Range, TextEdit};

use super::is_inside_html_comment;
use crate::ide::IdeContext;

use bindings::analyzed_template_binding_completions;
use components::builtin_component_completions;

pub(crate) use art::{complete_art, complete_inline_art};
pub(crate) use component_cache::CachedComponentMetadata;
pub(crate) use component_meta::component_metadata;
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

    let is_template_expression =
        crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset);

    if let Some(tag_ctx) = tag_context::opening_tag_context_at_offset(&ctx.content, ctx.offset) {
        if !tag_ctx.inside_attribute_value {
            let mut items_vec = contextual_directive_completions(ctx);
            items_vec.extend(native::native_element_attribute_completions(ctx));
            items_vec.extend(component_meta::component_surface_completions(ctx));
            return prepare_open_tag_completions(ctx, &tag_ctx, items_vec);
        }

        if !is_template_expression {
            return Vec::new();
        }
    }

    if is_template_expression {
        return analyzed_template_binding_completions(ctx, true);
    }

    let mut items_vec = Vec::new();

    // Add Vue directives and built-in components for template text/tag contexts.
    items_vec.extend(contextual_directive_completions(ctx));
    items_vec.extend(builtin_component_completions());
    if ctx.state.lsp_features().legacy_vue2 {
        items_vec.extend(components::legacy_vue2_component_completions());
    }
    items_vec.extend(component_meta::component_surface_completions(ctx));

    // Add common template snippets
    items_vec.extend(bindings::template_snippets());

    items_vec
}

fn filter_open_tag_completions(
    items: Vec<CompletionItem>,
    current_token: &str,
) -> Vec<CompletionItem> {
    let prefix = current_token.trim();
    if prefix.is_empty() {
        return items;
    }

    items
        .into_iter()
        .filter(|item| open_tag_completion_matches_prefix(&item.label, prefix))
        .collect()
}

fn prepare_open_tag_completions(
    ctx: &IdeContext,
    tag_ctx: &tag_context::OpenTagContext,
    items: Vec<CompletionItem>,
) -> Vec<CompletionItem> {
    let items = filter_open_tag_completions(items, &tag_ctx.current_token);
    with_open_tag_text_edits(ctx, tag_ctx, items)
}

fn with_open_tag_text_edits(
    ctx: &IdeContext,
    tag_ctx: &tag_context::OpenTagContext,
    mut items: Vec<CompletionItem>,
) -> Vec<CompletionItem> {
    if tag_ctx.current_token.is_empty() {
        return items;
    }

    let range = range_from_offsets(&ctx.content, tag_ctx.current_token_start, ctx.offset);
    for item in &mut items {
        let insert_text = item
            .insert_text
            .take()
            .unwrap_or_else(|| item.label.clone());
        item.text_edit = Some(CompletionTextEdit::Edit(TextEdit {
            range,
            new_text: replacement_text_for_open_tag_completion(
                &tag_ctx.current_token,
                &insert_text,
            ),
        }));
    }

    items
}

fn replacement_text_for_open_tag_completion(prefix: &str, insert_text: &str) -> String {
    if prefix.starts_with("v-bind:")
        && !insert_text.starts_with("v-bind:")
        && !insert_text.starts_with(':')
    {
        return format!("v-bind:{insert_text}");
    }

    if prefix.starts_with(':')
        && !insert_text.starts_with(':')
        && !insert_text.starts_with("v-bind:")
    {
        return format!(":{insert_text}");
    }

    if prefix.starts_with("v-slot:")
        && !insert_text.starts_with("v-slot:")
        && !insert_text.starts_with('#')
    {
        return format!("v-slot:{insert_text}");
    }

    if prefix.starts_with('#')
        && !insert_text.starts_with('#')
        && !insert_text.starts_with("v-slot:")
    {
        return format!("#{insert_text}");
    }

    insert_text.to_string()
}

fn range_from_offsets(content: &str, start: usize, end: usize) -> Range {
    let (start_line, start_character) = crate::ide::offset_to_position(content, start);
    let (end_line, end_character) = crate::ide::offset_to_position(content, end);
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

fn open_tag_completion_matches_prefix(label: &str, prefix: &str) -> bool {
    if prefix.contains('=') {
        return false;
    }

    if prefix.starts_with('@') {
        return label.starts_with(prefix) || label == "v-on";
    }

    if prefix.starts_with(':') {
        return is_bind_completion_label(label);
    }

    if prefix.starts_with("v-bind:") {
        return is_bind_completion_label(label);
    }

    if let Some(slot_prefix) = prefix.strip_prefix('#') {
        return label == "#" || label == "v-slot" || label.starts_with(slot_prefix);
    }

    label.starts_with(prefix)
}

fn is_bind_completion_label(label: &str) -> bool {
    label == ":"
        || label == "v-bind"
        || (!label.starts_with('@') && !label.starts_with('#') && !label.starts_with("v-"))
}

pub(crate) fn corsa_template_completions(ctx: &IdeContext) -> Vec<CompletionItem> {
    let mut items = contextual_directive_completions(ctx);
    items.extend(component_meta::component_surface_completions(ctx));
    items.extend(legacy_vue2_template_completions(ctx));
    if let Some(tag_ctx) = tag_context::opening_tag_context_at_offset(&ctx.content, ctx.offset)
        && !tag_ctx.inside_attribute_value
    {
        return prepare_open_tag_completions(ctx, &tag_ctx, items);
    }
    items
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
