//! Template and Art completion providers.
//!
//! Handles completions for template directives, built-in components,
//! Art blocks, and variant blocks.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionResponse,
    Documentation, InsertTextFormat, MarkupContent, MarkupKind,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

use super::{
    is_inside_art_tag, is_inside_html_comment, is_inside_variant_tag, items,
    should_suggest_art_block, should_suggest_variant_block,
};
use crate::ide::IdeContext;

/// Get completions for template context.
pub(crate) fn complete_template(ctx: &IdeContext) -> Vec<CompletionItem> {
    // If cursor is inside an HTML comment, offer @vize: directive completions only
    if is_inside_html_comment(&ctx.content, ctx.offset) {
        return vize_directive_completions();
    }

    let mut items_vec = Vec::new();

    // Add Vue directives
    items_vec.extend(directive_completions());

    // Add built-in components
    items_vec.extend(builtin_component_completions());

    // Use vize_croquis for accurate scope analysis and type information
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) {
        if let Some(ref script_setup) = descriptor.script_setup {
            let mut analyzer = Analyzer::with_options(AnalyzerOptions {
                analyze_script: true,
                ..Default::default()
            });
            analyzer.analyze_script_setup(&script_setup.content);
            let croquis = analyzer.finish();

            // Add bindings with accurate type information
            for (name, binding_type) in croquis.bindings.iter() {
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

            // Add props with type information
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

            // Add reactive sources with special handling
            for source in croquis.reactivity.sources() {
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
    }

    // Add common template snippets
    items_vec.extend(template_snippets());

    items_vec
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
        items::component_item("Transition", "Animate enter/leave", "<Transition name=\"$1\">\n\t$0\n</Transition>"),
        items::component_item("TransitionGroup", "Animate list", "<TransitionGroup name=\"$1\" tag=\"$2\">\n\t$0\n</TransitionGroup>"),
        items::component_item("KeepAlive", "Cache components", "<KeepAlive>\n\t$0\n</KeepAlive>"),
        items::component_item("Teleport", "Teleport content", "<Teleport to=\"$1\">\n\t$0\n</Teleport>"),
        items::component_item("Suspense", "Async dependencies", "<Suspense>\n\t<template #default>\n\t\t$0\n\t</template>\n\t<template #fallback>\n\t\tLoading...\n\t</template>\n</Suspense>"),
        items::component_item("component", "Dynamic component", "<component :is=\"$1\" />"),
        items::component_item("slot", "Slot outlet", "<slot name=\"$1\">$0</slot>"),
        items::component_item("template", "Template fragment", "<template #$1>\n\t$0\n</template>"),
    ]
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

/// Art block completions at root level.
fn art_block_completions() -> Vec<CompletionItem> {
    vec![CompletionItem {
        label: "art".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Create Art block".to_string()),
        insert_text: Some(
            "<art title=\"$1\" component=\"$2\">\n\t<variant name=\"$3\" default>\n\t\t$0\n\t</variant>\n</art>".to_string()
        ),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Art Block**\n\nDefines a component gallery entry with metadata and variants.\n\n```vue\n<art title=\"Button\" component=\"./Button.vue\">\n  <variant name=\"Primary\" default>\n    <Button>Click</Button>\n  </variant>\n</art>\n```".to_string(),
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
                "<script setup lang=\"ts\">\nimport $1 from '$2'\n</script>".to_string(),
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
