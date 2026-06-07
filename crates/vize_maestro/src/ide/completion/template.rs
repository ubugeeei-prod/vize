//! Template and Art completion providers.
//!
//! Handles completions for template directives, built-in components,
//! Art blocks, and variant blocks.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use std::collections::{BTreeMap, BTreeSet};

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionResponse,
    Documentation, InsertTextFormat, MarkupContent, MarkupKind,
};
use vize_atelier_sfc::croquis::{
    SfcCroquisOptions, analyze_sfc_descriptor, analyze_sfc_descriptor_with_context_legacy_vue2,
    analyze_sfc_descriptor_with_context_options_api,
};
use vize_croquis::{Analyzer, AnalyzerOptions, ScopeKind};
use vize_relief::BindingType;

use super::{
    is_inside_art_tag, is_inside_html_comment, is_inside_variant_tag, items,
    should_suggest_art_block, should_suggest_variant_block,
};
use crate::ide::definition::helpers as definition_helpers;
use crate::ide::{IdeContext, is_component_tag, kebab_to_pascal, pascal_to_kebab};

/// Get completions for template context.
pub(crate) fn complete_template(ctx: &IdeContext) -> Vec<CompletionItem> {
    // If cursor is inside an HTML comment, offer @vize: directive completions only
    if is_inside_html_comment(&ctx.content, ctx.offset) {
        return vize_directive_completions();
    }

    // `$style.|` in a template expression should resolve to class names
    // declared in the SFC's `<style module>` blocks.
    if let Some(items) = css_module_class_completions(ctx) {
        return items;
    }

    let mut items_vec = Vec::new();

    // Add Vue directives
    items_vec.extend(contextual_directive_completions(ctx));

    // Add built-in components
    items_vec.extend(builtin_component_completions());
    if ctx.state.lsp_features().legacy_vue2 {
        items_vec.extend(legacy_vue2_component_completions());
    }
    items_vec.extend(component_surface_completions(ctx));

    if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
        items_vec.extend(template_snippets());
        return items_vec;
    }

    items_vec.extend(analyzed_template_binding_completions(ctx, true));

    // Add common template snippets
    items_vec.extend(template_snippets());

    items_vec
}

/// `<style module>` populates a template-scope `$style` object whose
/// properties are the declared class names. When the cursor sits at
/// `$style.|` we surface those names instead of the usual directive list.
fn css_module_class_completions(ctx: &IdeContext) -> Option<Vec<CompletionItem>> {
    let before = &ctx.content[..ctx.offset.min(ctx.content.len())];
    let trimmed = before.trim_end_matches([' ', '\t']);
    if !trimmed.ends_with("$style.") {
        return None;
    }
    let descriptor = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let mut classes = BTreeSet::new();
    for style in descriptor.styles.iter() {
        let attrs = &style.attrs;
        if !attrs.contains_key("module") {
            continue;
        }
        for class in extract_css_class_names(&style.content) {
            classes.insert(class);
        }
    }
    if classes.is_empty() {
        return None;
    }
    Some(
        classes
            .into_iter()
            .map(|name| {
                #[allow(clippy::disallowed_macros)]
                CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("CSS module class".to_string()),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("`{name}` from `<style module>`."),
                    })),
                    sort_text: Some(format!("0{name}")),
                    ..Default::default()
                }
            })
            .collect(),
    )
}

/// Extract class selector names from raw CSS content. Approximate but good
/// enough for completion — matches `.identifier` outside attribute selectors.
fn extract_css_class_names(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && is_class_ident_byte(bytes[end]) {
                end += 1;
            }
            if end > start {
                out.push(css[start..end].to_string());
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

fn is_class_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
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
        legacy_vue2_component_completions()
    }
}

/// Get completions for Art files (*.art.vue).
pub(crate) fn complete_art(ctx: &IdeContext) -> Option<CompletionResponse> {
    let mut items_vec = Vec::new();

    let content = &ctx.content;
    let offset = ctx.offset;
    let before_cursor = &content[..offset.min(content.len())];

    if is_inside_art_tag(before_cursor) {
        items_vec.extend(art_attribute_completions());
    } else if is_inside_variant_tag(before_cursor) {
        items_vec.extend(variant_attribute_completions());
    } else if should_suggest_art_block(before_cursor) {
        items_vec.extend(art_block_completions());
    } else if should_suggest_variant_block(before_cursor) {
        items_vec.extend(variant_block_completions());
    }

    items_vec.extend(art_script_completions());

    if items_vec.is_empty() {
        None
    } else {
        Some(CompletionResponse::Array(items_vec))
    }
}

/// Get completions for inline <art> blocks in regular .vue files.
pub(crate) fn complete_inline_art(ctx: &IdeContext) -> Option<CompletionResponse> {
    let mut items_vec = Vec::new();

    let content = &ctx.content;
    let offset = ctx.offset;
    let before_cursor = &content[..offset.min(content.len())];

    if is_inside_art_tag(before_cursor) {
        items_vec.extend(art_attribute_completions());
    } else if is_inside_variant_tag(before_cursor) {
        items_vec.extend(variant_attribute_completions());
    } else if should_suggest_variant_block(before_cursor) {
        items_vec.extend(variant_block_completions());
        items_vec.push(self_component_completion());
    }

    if items_vec.is_empty() {
        None
    } else {
        Some(CompletionResponse::Array(items_vec))
    }
}

/// Vue directive completions.
pub(crate) fn directive_completions() -> Vec<CompletionItem> {
    vec![
        items::directive_item("v-if", "Conditional rendering", "v-if=\"$1\""),
        items::directive_item("v-else-if", "Else-if block", "v-else-if=\"$1\""),
        items::directive_item("v-else", "Else block", "v-else"),
        items::directive_item("v-for", "List rendering", "v-for=\"$1 in $2\" :key=\"$3\""),
        items::directive_item("v-on", "Event listener", "v-on:$1=\"$2\""),
        items::directive_item("v-bind", "Attribute binding", "v-bind:$1=\"$2\""),
        items::directive_item("v-model", "Two-way binding", "v-model=\"$1\""),
        items::directive_item("v-slot", "Named slot", "v-slot:$1"),
        items::directive_item("v-show", "Toggle visibility", "v-show=\"$1\""),
        items::directive_item("v-pre", "Skip compilation", "v-pre"),
        items::directive_item("v-once", "Render once", "v-once"),
        items::directive_item("v-memo", "Memoize subtree", "v-memo=\"[$1]\""),
        items::directive_item("v-cloak", "Hide until compiled", "v-cloak"),
        items::directive_item("v-text", "Set text content", "v-text=\"$1\""),
        items::directive_item("v-html", "Set innerHTML", "v-html=\"$1\""),
        items::directive_item("@", "Event shorthand", "@$1=\"$2\""),
        items::directive_item(":", "Bind shorthand", ":$1=\"$2\""),
        items::directive_item("#", "Slot shorthand", "#$1"),
    ]
}

/// Vue directive completions, extended with opt-in document-specific directives.
pub(crate) fn contextual_directive_completions(ctx: &IdeContext) -> Vec<CompletionItem> {
    let mut completions = directive_completions();
    if crate::utils::is_standalone_html_path(ctx.uri.path())
        && crate::utils::is_petite_vue_document(&ctx.content)
    {
        completions.extend(petite_vue_directive_completions());
    }
    completions
}

/// petite-vue directive and lifecycle event completions.
pub(crate) fn petite_vue_directive_completions() -> Vec<CompletionItem> {
    vec![
        petite_vue_item(
            "v-scope",
            "petite-vue scope root",
            "v-scope=\"{ $1 }\"",
            "Marks an HTML region controlled by petite-vue.",
        ),
        petite_vue_item(
            "v-effect",
            "Reactive inline effect",
            "v-effect=\"$1\"",
            "Runs reactive inline statements when referenced state changes.",
        ),
        petite_vue_item(
            "@vue:mounted",
            "petite-vue mounted event",
            "@vue:mounted=\"$1\"",
            "Listens for the petite-vue mounted lifecycle event.",
        ),
        petite_vue_item(
            "@vue:unmounted",
            "petite-vue unmounted event",
            "@vue:unmounted=\"$1\"",
            "Listens for the petite-vue unmounted lifecycle event.",
        ),
    ]
}

#[allow(clippy::disallowed_macros)]
fn petite_vue_item(
    label: &str,
    detail: &str,
    snippet: &str,
    documentation: &str,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some(detail.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**{}**\n\n{}\n\n[petite-vue](https://github.com/vuejs/petite-vue)",
                label, documentation
            ),
        })),
        ..Default::default()
    }
}

/// Vize directive completions for use inside HTML comments.
pub(crate) fn vize_directive_completions() -> Vec<CompletionItem> {
    vec![
        items::vize_directive_item(
            "@vize:todo",
            "@vize:todo $1 ",
            "TODO marker (warning in linter, stripped from build)",
        ),
        items::vize_directive_item(
            "@vize:fixme",
            "@vize:fixme $1 ",
            "FIXME marker (error in linter, stripped from build)",
        ),
        items::vize_directive_item(
            "@vize:expected",
            "@vize:expected",
            "Expect error on next line",
        ),
        items::vize_directive_item(
            "@vize:docs",
            "@vize:docs $1 ",
            "Documentation comment (stripped from build)",
        ),
        items::vize_directive_item(
            "@vize:ignore-start",
            "@vize:ignore-start",
            "Begin lint suppression region",
        ),
        items::vize_directive_item(
            "@vize:ignore-end",
            "@vize:ignore-end",
            "End lint suppression region",
        ),
        items::vize_directive_item(
            "@vize:level(warn)",
            "@vize:level($1)",
            "Override next-line diagnostic severity",
        ),
        items::vize_directive_item(
            "@vize:deprecated",
            "@vize:deprecated $1 ",
            "Deprecation warning",
        ),
        items::vize_directive_item("@vize:dev-only", "@vize:dev-only", "Strip in production"),
    ]
}

/// Built-in Vue component completions.
pub(crate) fn builtin_component_completions() -> Vec<CompletionItem> {
    vec![
        items::component_item(
            "Transition",
            "Animate enter/leave",
            "<Transition name=\"$1\">\n\t$0\n</Transition>",
        ),
        items::component_item(
            "TransitionGroup",
            "Animate list",
            "<TransitionGroup name=\"$1\" tag=\"$2\">\n\t$0\n</TransitionGroup>",
        ),
        items::component_item(
            "KeepAlive",
            "Cache components",
            "<KeepAlive>\n\t$0\n</KeepAlive>",
        ),
        items::component_item(
            "Teleport",
            "Teleport content",
            "<Teleport to=\"$1\">\n\t$0\n</Teleport>",
        ),
        items::component_item(
            "Suspense",
            "Async dependencies",
            "<Suspense>\n\t<template #default>\n\t\t$0\n\t</template>\n\t<template #fallback>\n\t\tLoading...\n\t</template>\n</Suspense>",
        ),
        items::component_item("component", "Dynamic component", "<component :is=\"$1\" />"),
        items::component_item("slot", "Slot outlet", "<slot name=\"$1\">$0</slot>"),
        items::component_item(
            "template",
            "Template fragment",
            "<template #$1>\n\t$0\n</template>",
        ),
    ]
}

fn legacy_vue2_component_completions() -> Vec<CompletionItem> {
    vec![
        items::component_item(
            "NuxtLink",
            "Nuxt 2 route link",
            "<NuxtLink to=\"$1\">$0</NuxtLink>",
        ),
        items::component_item(
            "nuxt-link",
            "Nuxt 2 route link",
            "<nuxt-link to=\"$1\">$0</nuxt-link>",
        ),
        items::component_item("Nuxt", "Nuxt 2 page outlet", "<Nuxt />"),
        items::component_item("NuxtChild", "Nuxt 2 child route outlet", "<NuxtChild />"),
        items::component_item(
            "ClientOnly",
            "Client-only render",
            "<ClientOnly>$0</ClientOnly>",
        ),
        items::component_item(
            "client-only",
            "Client-only render",
            "<client-only>$0</client-only>",
        ),
        items::component_item("NoSsr", "Client-only render", "<NoSsr>$0</NoSsr>"),
    ]
}

fn analyzed_template_binding_completions(
    ctx: &IdeContext,
    include_vue3_details: bool,
) -> Vec<CompletionItem> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
        return Vec::new();
    };

    // Parse the template so v-for / v-slot scopes land in the Croquis scope
    // chain. Without the AST, `analyze_sfc_descriptor` skips template-level
    // analysis and we lose nested binding visibility.
    let template_block = descriptor.template.as_ref();
    let allocator = vize_carton::Bump::new();
    let template_parse =
        template_block.map(|tb| (vize_armature::parse(&allocator, &tb.content), tb.loc.start));

    let croquis_options = SfcCroquisOptions::full();
    let croquis = if ctx.state.legacy_vue2_enabled() {
        analyze_sfc_descriptor_with_context_legacy_vue2(
            &descriptor,
            template_parse.as_ref().map(|((ast, _), _)| ast),
            croquis_options,
        )
        .croquis
    } else if ctx.state.options_api_enabled() {
        analyze_sfc_descriptor_with_context_options_api(
            &descriptor,
            template_parse.as_ref().map(|((ast, _), _)| ast),
            croquis_options,
        )
        .croquis
    } else {
        analyze_sfc_descriptor(
            &descriptor,
            template_parse.as_ref().map(|((ast, _), _)| ast),
            croquis_options,
        )
    };

    let mut items_vec = Vec::new();

    // Scope-aware completion: include bindings introduced by v-for / v-slot /
    // event-handler scopes that contain the cursor. Top-level setup bindings
    // are added by the loop below; we de-dup by name.
    if let Some((_, template_start)) = template_parse.as_ref() {
        let template_local = ctx.offset.saturating_sub(*template_start) as u32;
        for (name, _binding, scope_kind) in croquis.scopes.bindings_visible_at(template_local) {
            if !is_template_scope_kind(scope_kind) {
                continue;
            }
            if croquis.bindings.contains(name) {
                continue;
            }
            items_vec.push(template_scope_completion_item(name, scope_kind));
        }
    }
    let macro_prop_names: BTreeSet<&str> = if include_vue3_details {
        BTreeSet::new()
    } else {
        croquis
            .macros
            .props()
            .iter()
            .map(|prop| prop.name.as_str())
            .collect()
    };

    for (name, binding_type) in croquis.bindings.iter() {
        if !include_vue3_details
            && (!is_legacy_vue2_binding(binding_type)
                || binding_type == BindingType::Props && macro_prop_names.contains(name))
        {
            continue;
        }
        let (kind, type_detail, doc) = items::binding_type_to_completion_info(binding_type);
        #[allow(clippy::disallowed_macros)]
        items_vec.push(CompletionItem {
            label: name.to_string(),
            kind: Some(kind),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(type_detail.clone()),
                description: None,
            }),
            detail: Some(type_detail),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc,
            })),
            sort_text: Some(format!("0{}", name)),
            ..Default::default()
        });
    }

    if include_vue3_details {
        for prop in croquis.macros.props() {
            let prop_type = prop
                .prop_type
                .as_ref()
                .map(|t| t.as_str())
                .unwrap_or("unknown");
            let required = if prop.required { "" } else { "?" };

            #[allow(clippy::disallowed_macros)]
            items_vec.push(CompletionItem {
                label: prop.name.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(format!(": {}{}", prop_type, required)),
                    description: None,
                }),
                detail: Some(format!("prop: {}", prop_type)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**Prop** `{}`\n\n```typescript\n{}: {}{}\n```\n\n{}",
                        prop.name,
                        prop.name,
                        prop_type,
                        if prop.required { "" } else { " // optional" },
                        if prop.default_value.is_some() {
                            "Has default value"
                        } else {
                            ""
                        }
                    ),
                })),
                sort_text: Some(format!("0{}", prop.name)),
                ..Default::default()
            });
        }

        for source in croquis.reactivity.sources() {
            // Reactive bindings are already surfaced by the bindings loop
            // above; skip known bindings so an identifier is not offered twice.
            if croquis.bindings.contains(source.name.as_str()) {
                continue;
            }
            let kind_str = source.kind.to_display();
            #[allow(clippy::disallowed_macros)]
            items_vec.push(CompletionItem {
                label: source.name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(format!(" ({})", kind_str)),
                    description: None,
                }),
                detail: Some(format!("Reactive: {}", kind_str)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**{}** `{}`\n\n{}\n\nAuto-unwrapped in template.",
                        kind_str,
                        source.name,
                        if source.kind.needs_value_access() {
                            "Needs `.value` in script"
                        } else {
                            "Direct access (no `.value` needed)"
                        }
                    ),
                })),
                sort_text: Some(format!("0{}", source.name)),
                ..Default::default()
            });
        }
    }

    items_vec
}

/// True for scopes introduced by template-level constructs that bring new
/// names into expression scope (v-for, v-slot, event handlers, callbacks).
fn is_template_scope_kind(kind: ScopeKind) -> bool {
    matches!(
        kind,
        ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::EventHandler | ScopeKind::Callback
    )
}

#[allow(clippy::disallowed_macros)]
fn template_scope_completion_item(name: &str, scope_kind: ScopeKind) -> CompletionItem {
    let label = template_scope_label(scope_kind);
    CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::VARIABLE),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(format!(" ({label})")),
            description: Some("local".to_string()),
        }),
        detail: Some(format!("Local {label} binding")),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**Local** binding from `{label}` scope.\n\nVisible only inside the current `{label}` subtree."
            ),
        })),
        // Inner-scope bindings sort above setup-scope candidates that use
        // a plain `0` prefix.
        sort_text: Some(format!("00{name}")),
        ..Default::default()
    }
}

fn template_scope_label(kind: ScopeKind) -> &'static str {
    match kind {
        ScopeKind::VFor => "v-for",
        ScopeKind::VSlot => "v-slot",
        ScopeKind::EventHandler => "event handler",
        ScopeKind::Callback => "callback",
        _ => "local",
    }
}

fn is_legacy_vue2_binding(binding_type: BindingType) -> bool {
    matches!(
        binding_type,
        BindingType::Data | BindingType::Options | BindingType::Props | BindingType::VueGlobal
    )
}

/// Template snippet completions.
fn template_snippets() -> Vec<CompletionItem> {
    vec![
        items::snippet_item(
            "vfor",
            "v-for loop",
            "<$1 v-for=\"$2 in $3\" :key=\"$4\">\n\t$0\n</$1>",
        ),
        items::snippet_item("vif", "v-if block", "<$1 v-if=\"$2\">\n\t$0\n</$1>"),
        items::snippet_item("vshow", "v-show block", "<$1 v-show=\"$2\">\n\t$0\n</$1>"),
        items::snippet_item(
            "vmodel",
            "v-model input",
            "<input v-model=\"$1\" type=\"$2\" />",
        ),
        items::snippet_item("von", "v-on handler", "<$1 @$2=\"$3\">$0</$1>"),
        items::snippet_item("vbind", "v-bind attribute", "<$1 :$2=\"$3\">$0</$1>"),
    ]
}

fn component_surface_completions(ctx: &IdeContext) -> Vec<CompletionItem> {
    let Some(tag_ctx) = opening_tag_context_at_offset(&ctx.content, ctx.offset) else {
        return Vec::new();
    };

    if tag_ctx.inside_attribute_value {
        return Vec::new();
    }

    if tag_ctx.tag_name == "template" {
        if !is_slot_completion_prefix(&tag_ctx.current_token) {
            return Vec::new();
        }

        let Some(component_name) = nearest_open_component_before(&ctx.content, tag_ctx.tag_start)
        else {
            return Vec::new();
        };
        let Some(metadata) = component_metadata(ctx, &component_name) else {
            return Vec::new();
        };

        return metadata
            .slots
            .iter()
            .map(|slot| slot_completion_item(slot, &tag_ctx.current_token))
            .collect();
    }

    if !is_component_tag(&tag_ctx.tag_name) || !is_prop_completion_prefix(&tag_ctx.current_token) {
        return Vec::new();
    }

    let Some(metadata) = component_metadata(ctx, &tag_ctx.tag_name) else {
        return Vec::new();
    };
    let dynamic = is_dynamic_prop_prefix(&tag_ctx.current_token);

    metadata
        .props
        .iter()
        .map(|prop| prop_completion_item(prop, dynamic))
        .collect()
}

#[derive(Debug)]
struct OpenTagContext {
    tag_name: String,
    tag_start: usize,
    current_token: String,
    inside_attribute_value: bool,
}

fn opening_tag_context_at_offset(content: &str, offset: usize) -> Option<OpenTagContext> {
    let cursor = offset.min(content.len());
    let tag_start = content[..cursor].rfind('<')?;
    if content[tag_start..cursor].contains('>') {
        return None;
    }

    let bytes = content.as_bytes();
    let name_start = tag_start + 1;
    if matches!(bytes.get(name_start), Some(b'/' | b'!' | b'?')) {
        return None;
    }

    let mut name_end = name_start;
    while name_end < content.len() {
        let byte = bytes[name_end];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            name_end += 1;
        } else {
            break;
        }
    }

    if name_start == name_end || cursor <= name_end {
        return None;
    }

    let tag_name = content[name_start..name_end].to_string();
    let inside_attribute_value = is_inside_open_tag_attribute_value(content, tag_start, cursor);
    let current_token = current_open_tag_token(content, tag_start, cursor);

    Some(OpenTagContext {
        tag_name,
        tag_start,
        current_token,
        inside_attribute_value,
    })
}

fn is_inside_open_tag_attribute_value(content: &str, tag_start: usize, cursor: usize) -> bool {
    let mut quote = None;
    let mut pos = tag_start;

    while pos < cursor {
        let Some(ch) = content[pos..].chars().next() else {
            break;
        };
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        }
        pos += ch.len_utf8();
    }

    quote.is_some()
}

fn current_open_tag_token(content: &str, tag_start: usize, cursor: usize) -> String {
    let slice = &content[tag_start..cursor];
    let mut token_start = tag_start;

    for (relative, ch) in slice.char_indices() {
        if ch.is_ascii_whitespace() || ch == '<' {
            token_start = tag_start + relative + ch.len_utf8();
        }
    }

    content[token_start..cursor].trim_start().to_string()
}

fn is_prop_completion_prefix(prefix: &str) -> bool {
    prefix.is_empty()
        || is_dynamic_prop_prefix(prefix)
        || (!prefix.starts_with('@')
            && !prefix.starts_with('#')
            && !prefix.starts_with("v-")
            && !prefix.contains('='))
}

fn is_dynamic_prop_prefix(prefix: &str) -> bool {
    prefix.starts_with(':') || prefix.starts_with("v-bind:")
}

fn is_slot_completion_prefix(prefix: &str) -> bool {
    prefix.is_empty() || prefix.starts_with('#') || prefix.starts_with("v-slot:")
}

#[derive(Debug, Clone)]
pub(crate) struct ComponentMetadata {
    props: Vec<ComponentProp>,
    slots: Vec<ComponentSlot>,
}

/// Cached, parsed metadata for an imported component file, keyed in
/// [`crate::server::ServerState`] by resolved path. The `len` + `modified`
/// file stamp invalidates the entry when the component file changes on disk,
/// so completion doesn't re-read + re-parse + re-analyze the same component on
/// every keystroke inside an opening tag.
#[derive(Clone)]
pub(crate) struct CachedComponentMetadata {
    pub len: u64,
    pub modified: Option<std::time::SystemTime>,
    pub metadata: std::sync::Arc<ComponentMetadata>,
}

#[derive(Debug, Clone)]
struct ComponentProp {
    name: String,
    type_detail: Option<String>,
    required: bool,
    /// Default value source — populated from `withDefaults` or per-prop
    /// `default` config. Renders into the completion documentation so the
    /// user knows what the prop falls back to.
    default_value: Option<String>,
}

#[derive(Debug, Clone)]
struct InferredProp {
    type_detail: String,
    required: bool,
}

#[derive(Debug, Clone)]
struct ComponentSlot {
    name: String,
    props_type: Option<String>,
}

fn component_metadata(
    ctx: &IdeContext,
    component_name: &str,
) -> Option<std::sync::Arc<ComponentMetadata>> {
    let mut names = vec![component_name.to_string()];
    let pascal = kebab_to_pascal(component_name);
    if !names.iter().any(|name| name == &pascal) {
        names.push(pascal);
    }

    for name in names {
        let Some(import_path) = definition_helpers::find_import_path(ctx, &name) else {
            continue;
        };
        let resolved = definition_helpers::resolve_import_path(ctx.uri, &import_path)?;
        return cached_component_metadata(ctx, &resolved);
    }

    if let Some(import_path) = art_component_path(ctx, component_name) {
        let resolved = definition_helpers::resolve_import_path(ctx.uri, &import_path)?;
        return cached_component_metadata(ctx, &resolved);
    }

    None
}

/// Return parsed metadata for the component at `resolved`, reusing a cached
/// parse when the file's length + modification time are unchanged. Only the
/// `fs::metadata` stat runs on the hot (cache-hit) path; the disk read, SFC
/// parse, and Croquis analysis happen solely on a miss.
fn cached_component_metadata(
    ctx: &IdeContext,
    resolved: &std::path::Path,
) -> Option<std::sync::Arc<ComponentMetadata>> {
    let cache = ctx.state.component_metadata_cache();
    let (len, modified) = std::fs::metadata(resolved)
        .map(|meta| (meta.len(), meta.modified().ok()))
        .unwrap_or((0, None));

    if let Some(entry) = cache.get(resolved)
        && entry.len == len
        && entry.modified == modified
    {
        return Some(entry.metadata.clone());
    }

    let component_content = std::fs::read_to_string(resolved).ok()?;
    let metadata = std::sync::Arc::new(extract_component_metadata(
        &component_content,
        &resolved.to_string_lossy(),
        ctx.state.options_api_enabled(),
        ctx.state.legacy_vue2_enabled(),
    ));
    cache.insert(
        resolved.to_path_buf(),
        CachedComponentMetadata {
            len,
            modified,
            metadata: metadata.clone(),
        },
    );
    Some(metadata)
}

fn art_component_path(ctx: &IdeContext<'_>, component_name: &str) -> Option<String> {
    if !ctx.uri.path().ends_with(".art.vue") {
        return None;
    }

    let allocator = vize_carton::Bump::new();
    let art_desc = vize_musea::parse_art(
        &allocator,
        &ctx.content,
        vize_musea::ArtParseOptions::default(),
    )
    .ok()?;
    let component_path = art_desc.metadata.component?;
    // Reuse the script_setup already extracted by `parse_art` above instead of
    // re-parsing the whole buffer with `parse_sfc` just to read the defineArt
    // component name.
    if let Some(script_setup) = art_desc.script_setup.as_ref()
        && let Some(defined_component) =
            crate::virtual_code::find_define_art_component_name(script_setup.content)
    {
        let pascal_component = kebab_to_pascal(component_name);
        if component_name == defined_component || pascal_component == defined_component {
            return Some(component_path.to_string());
        }
    }

    let stem = std::path::Path::new(component_path)
        .file_stem()
        .and_then(|stem| stem.to_str())?;

    let pascal_component = kebab_to_pascal(component_name);
    let pascal_stem = kebab_to_pascal(stem);
    (component_name == stem || pascal_component == pascal_stem).then(|| component_path.to_string())
}

fn extract_component_metadata(
    content: &str,
    filename: &str,
    options_api: bool,
    legacy_vue2: bool,
) -> ComponentMetadata {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: filename.to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return ComponentMetadata {
            props: Vec::new(),
            slots: Vec::new(),
        };
    };

    let mut props = Vec::new();
    let mut slots = Vec::new();
    let mut seen_props = BTreeSet::new();
    let mut seen_slots = BTreeSet::new();

    if let Some(script_content) = descriptor
        .script_setup
        .as_ref()
        .map(|script| script.content.as_ref())
        .or_else(|| {
            descriptor
                .script
                .as_ref()
                .map(|script| script.content.as_ref())
        })
    {
        let analyzer_options = AnalyzerOptions {
            analyze_script: true,
            ..Default::default()
        };
        let mut analyzer = Analyzer::with_options(analyzer_options);
        if legacy_vue2 {
            analyzer = analyzer.with_legacy_vue2();
        } else if options_api {
            analyzer = analyzer.with_options_api();
        }
        if descriptor.script_setup.is_some() {
            analyzer.analyze_script_setup(script_content);
        } else {
            analyzer.analyze_script_plain(script_content);
        }
        let summary = analyzer.finish();
        let inferred_prop_types = infer_define_props_type_map(script_content);

        for prop in summary.macros.props() {
            if seen_props.insert(prop.name.to_string()) {
                let inferred = inferred_prop_types.get(prop.name.as_str());
                props.push(ComponentProp {
                    name: prop.name.to_string(),
                    type_detail: prop
                        .prop_type
                        .as_ref()
                        .map(|ty| ty.to_string())
                        .or_else(|| inferred.map(|prop| prop.type_detail.clone())),
                    required: inferred.map_or(prop.required, |prop| prop.required),
                    default_value: prop.default_value.as_ref().map(|d| d.to_string()),
                });
            }
        }

        // defineModel<T>() introduces a prop alongside an `update:NAME`
        // event. Prop completion only knew about defineProps before, so
        // child components using defineModel showed no prop suggestions.
        // See #686.
        for model in summary.macros.models() {
            if seen_props.insert(model.name.to_string()) {
                props.push(ComponentProp {
                    name: model.name.to_string(),
                    type_detail: model.model_type.as_ref().map(|ty| ty.to_string()),
                    required: model.required,
                    default_value: model.default_value.as_ref().map(|d| d.to_string()),
                });
            }
        }

        if legacy_vue2 {
            for (name, binding_type) in summary.bindings.iter() {
                if binding_type == BindingType::Props && seen_props.insert(name.to_string()) {
                    props.push(ComponentProp {
                        name: name.to_string(),
                        type_detail: None,
                        required: false,
                        default_value: None,
                    });
                }
            }
        }

        for (name, prop) in inferred_prop_types {
            if seen_props.insert(name.clone()) {
                props.push(ComponentProp {
                    name,
                    type_detail: Some(prop.type_detail),
                    required: prop.required,
                    default_value: None,
                });
            }
        }

        for slot in summary.macros.slots() {
            let name = slot.name.to_string();
            if seen_slots.insert(name.clone()) {
                slots.push(ComponentSlot {
                    name,
                    props_type: slot.props_type.as_ref().map(|props| props.to_string()),
                });
            }
        }
    }

    if let Some(template) = descriptor.template.as_ref() {
        for slot in extract_template_slot_outlets(template.content.as_ref()) {
            if seen_slots.insert(slot.name.clone()) {
                slots.push(slot);
            }
        }
    }

    ComponentMetadata { props, slots }
}

fn prop_completion_item(prop: &ComponentProp, dynamic: bool) -> CompletionItem {
    let kebab_name = pascal_to_kebab(&prop.name);
    let label = if dynamic {
        prop.name.clone()
    } else {
        kebab_name.clone()
    };
    let insert_name = label.clone();
    let insert_text = if !dynamic && prop.type_detail.as_deref() == Some("boolean") {
        insert_name
    } else {
        format!("{insert_name}=\"$1\"")
    };
    let required = if prop.required {
        "required"
    } else {
        "optional"
    };
    let type_detail = prop.type_detail.as_deref().unwrap_or("unknown");

    let mut doc_body = format!(
        "**Prop** `{}`\n\n```typescript\n{}: {}\n```",
        prop.name, prop.name, type_detail
    );
    if let Some(ref default) = prop.default_value {
        doc_body.push_str("\n\nDefault: `");
        doc_body.push_str(default);
        doc_body.push('`');
    }

    CompletionItem {
        label,
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(format!("prop: {type_detail} ({required})")),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(format!(": {type_detail}")),
            description: Some(required.to_string()),
        }),
        insert_text: Some(insert_text),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc_body,
        })),
        sort_text: Some(format!("00-prop-{kebab_name}")),
        ..Default::default()
    }
}

fn slot_completion_item(slot: &ComponentSlot, prefix: &str) -> CompletionItem {
    let after_hash = prefix.starts_with('#');
    let after_v_slot = prefix.starts_with("v-slot:");
    let label = if after_hash || after_v_slot {
        slot.name.clone()
    } else {
        format!("#{}", slot.name)
    };
    let destructure = slot
        .props_type
        .as_ref()
        .and_then(|ty| extract_slot_prop_names(ty));
    let value_snippet = match destructure {
        Some(names) if !names.is_empty() => {
            // Pre-populate the destructure with the resolved slot prop names
            // so the user gets `{ row, col }` rather than just `="$1"`.
            // The `${1:...}` placeholder lets the editor select the names.
            format!("{{ ${{1:{}}} }}", names.join(", "))
        }
        _ => "$1".to_string(),
    };
    let insert_text = if slot.props_type.is_some() {
        if after_hash || after_v_slot {
            format!("{}=\"{value_snippet}\"", slot.name)
        } else {
            format!("#{}=\"{value_snippet}\"", slot.name)
        }
    } else if after_hash || after_v_slot {
        slot.name.clone()
    } else {
        format!("#{}", slot.name)
    };

    CompletionItem {
        label,
        kind: Some(CompletionItemKind::FIELD),
        detail: Some(
            slot.props_type
                .as_ref()
                .map(|props| format!("slot props: {props}"))
                .unwrap_or_else(|| "slot".to_string()),
        ),
        insert_text: Some(insert_text),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: slot.props_type.as_ref().map(|props| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Slot** `{}`\n\n```typescript\n{}\n```", slot.name, props),
            })
        }),
        sort_text: Some(format!("00-slot-{}", slot.name)),
        ..Default::default()
    }
}

/// Extract slot prop names from a TS function-shape type like
/// `(props: { foo: T; bar: U }): any` or `{ foo: T; bar: U }`. Returns the
/// names in source order. The extractor is approximate — it stops at the
/// first `{` and reads property names up to `:` — but it's enough to
/// pre-populate a slot destructure for editor convenience.
fn extract_slot_prop_names(ts_type: &str) -> Option<Vec<String>> {
    let brace_start = ts_type.find('{')?;
    let body = &ts_type[brace_start + 1..];
    let mut depth: i32 = 0;
    let mut name = String::new();
    let mut waiting_for_colon = false;
    let mut names = Vec::new();
    for ch in body.chars() {
        match ch {
            '{' | '<' | '(' | '[' => depth += 1,
            '}' if depth == 0 => break,
            '}' | '>' | ')' | ']' => depth -= 1,
            _ => {}
        }
        if depth != 0 {
            continue;
        }
        if !waiting_for_colon {
            if ch == ':' {
                let trimmed = name.trim();
                if !trimmed.is_empty()
                    && trimmed
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
                {
                    names.push(trimmed.to_string());
                }
                name.clear();
                waiting_for_colon = true;
                continue;
            }
            if ch == ';' || ch == ',' || ch == '\n' {
                name.clear();
                continue;
            }
            name.push(ch);
        } else if ch == ';' || ch == ',' || ch == '\n' {
            waiting_for_colon = false;
        }
    }
    if names.is_empty() { None } else { Some(names) }
}

fn nearest_open_component_before(content: &str, before_offset: usize) -> Option<String> {
    let before = &content[..before_offset.min(content.len())];
    let mut stack = Vec::new();
    let mut pos = 0usize;

    while let Some(relative_start) = before[pos..].find('<') {
        let tag_start = pos + relative_start;
        if before[tag_start..].starts_with("<!--") {
            let Some(end) = before[tag_start + 4..].find("-->") else {
                break;
            };
            pos = tag_start + 4 + end + 3;
            continue;
        }

        let Some(tag_end) = find_tag_end(before, tag_start) else {
            break;
        };
        let tag = &before[tag_start..=tag_end];
        let name_start = tag_start + if tag.starts_with("</") { 2 } else { 1 };
        if matches!(before.as_bytes().get(name_start), Some(b'!' | b'?')) {
            pos = tag_end + 1;
            continue;
        }

        let name_end = read_tag_name_end(before, name_start);
        if name_start == name_end {
            pos = tag_end + 1;
            continue;
        }

        let tag_name = &before[name_start..name_end];
        if tag.starts_with("</") {
            if let Some(index) = stack.iter().rposition(|open: &String| open == tag_name) {
                stack.truncate(index);
            }
        } else if is_component_tag(tag_name) && !is_self_closing_tag(tag) {
            stack.push(tag_name.to_string());
        }

        pos = tag_end + 1;
    }

    stack.pop()
}

fn find_tag_end(content: &str, tag_start: usize) -> Option<usize> {
    let mut quote = None;
    let mut pos = tag_start;

    while pos < content.len() {
        let ch = content[pos..].chars().next()?;
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '>' {
            return Some(pos);
        }
        pos += ch.len_utf8();
    }

    None
}

fn read_tag_name_end(content: &str, mut pos: usize) -> usize {
    while pos < content.len() {
        let byte = content.as_bytes()[pos];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            pos += 1;
        } else {
            break;
        }
    }
    pos
}

fn is_self_closing_tag(tag: &str) -> bool {
    tag.trim_end_matches('>').trim_end().ends_with('/')
}

fn infer_define_props_type_map(script: &str) -> BTreeMap<String, InferredProp> {
    let mut props = BTreeMap::new();
    let mut search_start = 0usize;

    while let Some(relative) = script[search_start..].find("defineProps") {
        let name_start = search_start + relative;
        let after_name = name_start + "defineProps".len();
        let mut pos = skip_ws(script, after_name);
        if script.as_bytes().get(pos) != Some(&b'<') {
            search_start = after_name;
            continue;
        }

        let Some((type_arg, end)) = extract_balanced_after(script, pos, '<', '>') else {
            search_start = after_name;
            continue;
        };
        pos = end;

        let type_arg = type_arg.trim();
        if let Some(body) = braced_body(type_arg) {
            for member in parse_type_literal_members(body) {
                if let Some((name, optional, type_detail)) = parse_member_name_and_type(member) {
                    props.insert(
                        name,
                        InferredProp {
                            type_detail,
                            required: !optional,
                        },
                    );
                }
            }
        }

        search_start = pos;
    }

    props
}

fn extract_template_slot_outlets(template: &str) -> Vec<ComponentSlot> {
    let mut slots = Vec::new();
    let mut seen = BTreeSet::new();
    let mut pos = 0usize;

    while let Some(relative_start) = template[pos..].find("<slot") {
        let tag_start = pos + relative_start;
        let after_name = tag_start + "<slot".len();
        if template
            .as_bytes()
            .get(after_name)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            pos = after_name;
            continue;
        }

        let Some(tag_end) = find_tag_end(template, tag_start) else {
            break;
        };
        let tag = &template[tag_start..=tag_end];
        let name = find_attr_value(tag, "name").unwrap_or_else(|| "default".to_string());
        if seen.insert(name.clone()) {
            slots.push(ComponentSlot {
                name,
                props_type: None,
            });
        }
        pos = tag_end + 1;
    }

    slots
}

fn parse_type_literal_members(body: &str) -> Vec<&str> {
    let mut members = Vec::new();
    let mut start = 0usize;
    let mut state = SplitState::default();

    for (idx, ch) in body.char_indices() {
        state.accept(ch, body, idx);
        if state.depth == 0 && state.quote.is_none() && matches!(ch, ';' | ',' | '\n') {
            let member = body[start..idx].trim();
            if !member.is_empty() {
                members.push(member);
            }
            start = idx + ch.len_utf8();
        }
    }

    let member = body[start..].trim();
    if !member.is_empty() {
        members.push(member);
    }

    members
}

#[derive(Default)]
struct SplitState {
    depth: i32,
    quote: Option<char>,
}

impl SplitState {
    fn accept(&mut self, ch: char, source: &str, idx: usize) {
        if let Some(quote) = self.quote {
            if ch == quote && !is_escaped(source, idx) {
                self.quote = None;
            }
            return;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            self.quote = Some(ch);
        } else if matches!(ch, '{' | '[' | '(' | '<') {
            self.depth += 1;
        } else if matches!(ch, '}' | ']' | ')')
            || (ch == '>' && !previous_non_ws_is(source, idx, '='))
        {
            self.depth = self.depth.saturating_sub(1);
        }
    }
}

fn parse_member_name_and_type(member: &str) -> Option<(String, bool, String)> {
    let member = strip_readonly(member.trim());
    let (name, name_end) = parse_type_member_name(member)?;
    let rest = member[name_end..].trim_start();
    let (optional, rest) = if let Some(rest) = rest.strip_prefix('?') {
        (true, rest.trim_start())
    } else {
        (false, rest)
    };
    let type_detail = rest.strip_prefix(':')?.trim();
    if type_detail.is_empty() {
        return None;
    }

    Some((name, optional, type_detail.to_string()))
}

fn parse_type_member_name(member: &str) -> Option<(String, usize)> {
    let mut chars = member.char_indices();
    let (_, first) = chars.next()?;
    if first == '"' || first == '\'' {
        for (idx, ch) in chars {
            if ch == first && !is_escaped(member, idx) {
                return Some((member[1..idx].to_string(), idx + ch.len_utf8()));
            }
        }
        return None;
    }

    let mut end = 0usize;
    for (idx, ch) in member.char_indices() {
        if idx == 0 {
            if !(ch == '_' || ch == '$' || ch.is_ascii_alphabetic()) {
                return None;
            }
        } else if !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric() || ch == '-') {
            break;
        }
        end = idx + ch.len_utf8();
    }

    (end > 0).then(|| (member[..end].to_string(), end))
}

fn strip_readonly(value: &str) -> &str {
    value
        .strip_prefix("readonly ")
        .map(str::trim_start)
        .unwrap_or(value)
}

fn extract_balanced_after(
    source: &str,
    open_offset: usize,
    open: char,
    close: char,
) -> Option<(&str, usize)> {
    if !source[open_offset..].starts_with(open) {
        return None;
    }

    let mut depth = 0i32;
    let mut quote = None;
    let content_start = open_offset + open.len_utf8();
    let mut pos = open_offset;

    while pos < source.len() {
        let ch = source[pos..].chars().next()?;
        if let Some(open_quote) = quote {
            if ch == open_quote && !is_escaped(source, pos) {
                quote = None;
            }
            pos += ch.len_utf8();
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            quote = Some(ch);
        } else if ch == open {
            depth += 1;
        } else if ch == close && !(close == '>' && previous_non_ws_is(source, pos, '=')) {
            depth -= 1;
            if depth == 0 {
                return Some((&source[content_start..pos], pos + ch.len_utf8()));
            }
        }

        pos += ch.len_utf8();
    }

    None
}

fn braced_body(value: &str) -> Option<&str> {
    let start = skip_ws(value, 0);
    if value.as_bytes().get(start) != Some(&b'{') {
        return None;
    }
    extract_balanced_after(value, start, '{', '}').map(|(body, _)| body)
}

fn find_attr_value(tag: &str, attr: &str) -> Option<String> {
    let mut pos = 0usize;
    while let Some(relative) = tag[pos..].find(attr) {
        let start = pos + relative;
        let end = start + attr.len();
        let boundary_before = start == 0
            || tag
                .as_bytes()
                .get(start - 1)
                .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_');
        let boundary_after = tag
            .as_bytes()
            .get(end)
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_');
        if !boundary_before || !boundary_after {
            pos = end;
            continue;
        }

        let mut value_start = skip_ws(tag, end);
        if tag.as_bytes().get(value_start) != Some(&b'=') {
            pos = end;
            continue;
        }
        value_start = skip_ws(tag, value_start + 1);
        let quote = tag.as_bytes().get(value_start).copied()?;
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        let value_content_start = value_start + 1;
        let value_end = tag[value_content_start..].find(quote as char)? + value_content_start;
        return Some(tag[value_content_start..value_end].to_string());
    }

    None
}

fn skip_ws(source: &str, mut pos: usize) -> usize {
    while pos < source.len() {
        let byte = source.as_bytes()[pos];
        if byte.is_ascii_whitespace() {
            pos += 1;
        } else {
            break;
        }
    }
    pos
}

fn is_escaped(source: &str, idx: usize) -> bool {
    let mut count = 0usize;
    let mut pos = idx;
    while pos > 0 && source.as_bytes()[pos - 1] == b'\\' {
        count += 1;
        pos -= 1;
    }
    count % 2 == 1
}

fn previous_non_ws_is(source: &str, idx: usize, expected: char) -> bool {
    source[..idx].chars().rev().find(|ch| !ch.is_whitespace()) == Some(expected)
}

/// Art block completions at root level.
fn art_block_completions() -> Vec<CompletionItem> {
    vec![CompletionItem {
        label: "art".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Create Art block".to_string()),
        insert_text: Some(
            "<art>\n\t<variant name=\"$1\" default>\n\t\t$0\n\t</variant>\n</art>".to_string()
        ),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Art Block**\n\nDefines component variants. Metadata and target component are declared with `defineArt` in `<script setup>`.\n\n```vue\n<script setup lang=\"ts\">\ndefineArt(\"./Button.vue\", { title: \"Button\" });\n</script>\n\n<art>\n  <variant name=\"Primary\" default>\n    <Button>Click</Button>\n  </variant>\n</art>\n```".to_string(),
        })),
        ..Default::default()
    }]
}

/// Art attribute completions inside <art> tag.
fn art_attribute_completions() -> Vec<CompletionItem> {
    vec![
        items::attr_item("title", "Component title (required)", "title=\"$1\""),
        items::attr_item("component", "Path to component file", "component=\"$1\""),
        items::attr_item("description", "Component description", "description=\"$1\""),
        items::attr_item(
            "category",
            "Component category (e.g., atoms, molecules)",
            "category=\"$1\"",
        ),
        items::attr_item("tags", "Comma-separated tags", "tags=\"$1\""),
        items::attr_item(
            "status",
            "Component status (ready, draft, deprecated)",
            "status=\"$1\"",
        ),
        items::attr_item("order", "Display order in gallery", "order=\"$1\""),
    ]
}

/// Variant block completions inside <art>.
fn variant_block_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "variant".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Create variant block".to_string()),
            insert_text: Some("<variant name=\"$1\">\n\t$0\n</variant>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**Variant Block**\n\nDefines a component variation with specific props.\n\n```vue\n<variant name=\"Primary\" default>\n  <Button variant=\"primary\">Click</Button>\n</variant>\n```".to_string(),
            })),
            ..Default::default()
        },
        CompletionItem {
            label: "variant with args".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Create variant with args".to_string()),
            insert_text: Some(
                "<variant name=\"$1\" args='{\"$2\": $3}'>\n\t$0\n</variant>".to_string(),
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Variant attribute completions inside <variant> tag.
fn variant_attribute_completions() -> Vec<CompletionItem> {
    vec![
        items::attr_item("name", "Variant name (required)", "name=\"$1\""),
        items::attr_item("default", "Mark as default variant", "default"),
        items::attr_item("args", "Props as JSON", "args='{\"$1\": $2}'"),
        items::attr_item(
            "viewport",
            "Viewport dimensions (WxH or WxH@scale)",
            "viewport=\"$1\"",
        ),
        items::attr_item("skip-vrt", "Skip visual regression test", "skip-vrt"),
    ]
}

/// Completion item for <Self> component reference in inline art blocks.
fn self_component_completion() -> CompletionItem {
    CompletionItem {
        label: "Self".to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some("Reference to the host component".to_string()),
        insert_text: Some("<Self $1>$0</Self>".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**`<Self>`**\n\nReferences the host component in inline art blocks.\nReplaced with the component name at build time.".to_string(),
        })),
        ..Default::default()
    }
}

/// Script block completions for Art files.
fn art_script_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "script setup".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Add script setup block".to_string()),
            insert_text: Some(
                "<script setup lang=\"ts\">\ndefineArt(\"$1\", {\n\ttitle: \"$2\",\n});\n</script>"
                    .to_string(),
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "style".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Add style block".to_string()),
            insert_text: Some("<style scoped>\n$0\n</style>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

#[cfg(test)]
mod cache_tests {
    use super::cached_component_metadata;
    use crate::ide::IdeContext;
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Url;

    #[test]
    fn component_metadata_cache_hits_then_invalidates_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let component = dir.path().join("Widget.vue");
        std::fs::write(
            &component,
            "<script setup lang=\"ts\">\nconst props = defineProps<{ a: string }>()\n</script>\n",
        )
        .unwrap();

        let state = ServerState::new();
        let uri = Url::parse("file:///host.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<template></template>".to_string(),
            1,
            "vue".to_string(),
        );
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();

        let first = cached_component_metadata(&ctx, &component).unwrap();
        let second = cached_component_metadata(&ctx, &component).unwrap();
        assert!(
            std::sync::Arc::ptr_eq(&first, &second),
            "an unchanged component file should hit the cache (same Arc, no re-parse)",
        );
        let first_prop_count = first.props.len();

        // Rewrite with a different length so the file stamp changes; the next
        // lookup must recompute rather than serve the stale cached parse.
        std::fs::write(
            &component,
            "<script setup lang=\"ts\">\nconst props = defineProps<{ a: string; bb: number }>()\n</script>\n",
        )
        .unwrap();

        let third = cached_component_metadata(&ctx, &component).unwrap();
        assert!(
            !std::sync::Arc::ptr_eq(&first, &third),
            "a changed component file must invalidate the cached entry",
        );
        assert!(
            third.props.len() > first_prop_count,
            "recomputed metadata should reflect the added prop ({} -> {})",
            first_prop_count,
            third.props.len(),
        );
    }
}

#[cfg(test)]
mod options_api_tests {
    use super::analyzed_template_binding_completions;
    use crate::ide::IdeContext;
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Url;

    const OPTIONS_API_SFC: &str = "<script>\nexport default {\n  data() {\n    return { greeting: 'hello' }\n  },\n}\n</script>\n<template>\n  <p>{{ greeting }}</p>\n</template>\n";

    fn binding_labels(options_api: bool) -> Vec<String> {
        let state = ServerState::new();
        if options_api {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(
                dir.path().join("vize.config.json"),
                r#"{ "typeChecker": { "optionsApi": true } }"#,
            )
            .unwrap();
            state.load_workspace_config(dir.path());
        }
        let uri = Url::parse("file:///comp.vue").unwrap();
        state.documents.open(
            uri.clone(),
            OPTIONS_API_SFC.to_string(),
            1,
            "vue".to_string(),
        );
        let offset = OPTIONS_API_SFC.find("greeting }}").unwrap();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        analyzed_template_binding_completions(&ctx, true)
            .into_iter()
            .map(|item| item.label)
            .collect()
    }

    #[test]
    fn options_api_data_binding_completed_when_enabled() {
        let labels = binding_labels(true);
        assert!(
            labels.iter().any(|label| label == "greeting"),
            "the Options API data() binding should be offered as a template completion \
             when optionsApi is enabled; got {labels:?}"
        );
    }

    #[test]
    fn options_api_data_binding_absent_by_default() {
        let labels = binding_labels(false);
        assert!(
            !labels.iter().any(|label| label == "greeting"),
            "without optionsApi the Options API data() binding must not resolve \
             (opt-in keeps the default <script setup> path zero cost); got {labels:?}"
        );
    }
}
