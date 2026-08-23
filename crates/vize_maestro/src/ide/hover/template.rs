//! Template hover provider.
//!
//! Provides hover information for template expressions, Vue directives,
//! and template bindings from script setup.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::Hover;
use vize_croquis::{Drawer, DrawerOptions};
use vize_relief::BindingType;

use super::{HoverBuilder, HoverService};
use crate::ide::{IdeContext, template_scope};

impl HoverService {
    /// Get hover for template context.
    pub(super) fn hover_template(ctx: &IdeContext) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        if let Some(hover) = Self::hover_component_tag(ctx) {
            return Some(hover);
        }

        if let Some(hover) = super::component_prop::hover_attribute(ctx) {
            return Some(hover);
        }

        if let Some(hover) = Self::hover_directive(&word) {
            return Some(hover);
        }

        #[cfg(feature = "native")]
        if let Some(hover) = Self::hover_native_dom_attribute(ctx) {
            return Some(hover);
        }

        #[cfg(feature = "native")]
        if let Some(hover) = Self::hover_native_dom_tag(ctx) {
            return Some(hover);
        }

        if let Some(hover) = Self::hover_template_directive_attribute(ctx) {
            return Some(hover);
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
            return None;
        }

        if let Some(hover) = Self::hover_petite_vue_scope_binding(ctx, &word) {
            return Some(hover);
        }

        if let Some(hover) = template_scope::v_for_hover(ctx, &word) {
            return Some(hover);
        }
        if let Some(hover) = super::backend::binding_type_hover(ctx, &word) {
            return Some(hover);
        }

        if let Some(type_info) = super::backend::heuristic_type_at(ctx) {
            #[allow(clippy::disallowed_macros)]
            let signature = format!("{word}: {}", type_info.display);
            let mut builder = HoverBuilder::new()
                .title(&word)
                .meta("Template expression type")
                .code("typescript", &signature);

            if let Some(ref doc) = type_info.documentation {
                builder = builder.section("Documentation", doc);
            }

            return Some(builder.build());
        }

        if let Some(ref virtual_docs) = ctx.virtual_docs
            && let Some(ref script_setup) = virtual_docs.script_setup
        {
            let bindings =
                crate::virtual_code::extract_simple_bindings(&script_setup.content, true);
            if bindings.contains(&word) {
                return Some(
                    HoverBuilder::new()
                        .title(&word)
                        .meta("Template binding")
                        .description("Binding from `<script setup>`.")
                        .bullets(
                            "Behavior",
                            &[
                                "Available directly in the template scope.",
                                "Vue automatically unwraps refs when rendering templates.",
                            ],
                        )
                        .build(),
                );
            }
        }

        Some(
            HoverBuilder::new()
                .title(&word)
                .meta("Template expression")
                .description("Expression evaluated against the component template scope.")
                .build(),
        )
    }

    fn hover_template_directive_attribute(ctx: &IdeContext<'_>) -> Option<Hover> {
        let attr_name = template_attribute_name_at_offset(&ctx.content, ctx.offset)?;

        if let Some(event_name) = attr_name
            .strip_prefix('@')
            .or_else(|| attr_name.strip_prefix("v-on:"))
        {
            let event_name = event_name
                .split_once('.')
                .map_or(event_name, |(name, _)| name);
            let title = if event_name.is_empty() {
                "v-on".to_string()
            } else {
                format!("@{event_name}")
            };
            let example = if event_name.is_empty() {
                "v-on:event=\"handler\"".to_string()
            } else {
                format!("@{event_name}=\"handler\"")
            };

            return Some(
                HoverBuilder::new()
                    .title(&title)
                    .meta("Vue event listener")
                    .example("vue", &example)
                    .description(
                        "Attaches a DOM or component event listener. The handler expression is evaluated in component scope.",
                    )
                    .bullets(
                        "Template behavior",
                        &[
                            "`$event` is available inside inline handler expressions.",
                            "Event modifiers such as `.stop`, `.prevent`, and key modifiers are compiled by Vue.",
                        ],
                    )
                    .docs(
                        "Vue Event Handling",
                        "https://vuejs.org/guide/essentials/event-handling.html",
                    )
                    .build(),
            );
        }

        if attr_name.starts_with(':') || attr_name.starts_with("v-bind:") || attr_name == "v-bind" {
            return Some(
                HoverBuilder::new()
                    .title("v-bind")
                    .meta("Vue attribute / prop binding")
                    .example("vue", ":prop=\"expression\"")
                    .description(
                        "Binds an attribute or component prop to a JavaScript expression in template scope.",
                    )
                    .bullets(
                        "Template behavior",
                        &[
                            "Native element bindings patch DOM attributes or reflected properties.",
                            "Component bindings resolve to props when the target is a component.",
                        ],
                    )
                    .docs(
                        "Vue v-bind",
                        "https://vuejs.org/api/built-in-directives.html#v-bind",
                    )
                    .build(),
            );
        }

        if attr_name.starts_with('#') || attr_name.starts_with("v-slot:") || attr_name == "v-slot" {
            return Self::hover_directive("v-slot");
        }

        if attr_name.starts_with("v-") {
            let without_argument = attr_name
                .split_once(':')
                .map_or(attr_name, |(name, _)| name);
            let base = without_argument
                .split_once('.')
                .map_or(without_argument, |(name, _)| name);
            return Self::hover_directive(base);
        }

        None
    }

    /// Get hover for TypeScript binding using croquis analysis.
    pub(super) fn hover_ts_binding(ctx: &IdeContext, word: &str) -> Option<Hover> {
        // Parse SFC to get script content
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

        // Get the script content for type inference
        let script_content = descriptor
            .script_setup
            .as_ref()
            .map(|s| s.content.as_ref())
            .or_else(|| descriptor.script.as_ref().map(|s| s.content.as_ref()));

        // Create a drawer and analyze script.
        let drawer_options = DrawerOptions::full();
        let mut drawer = Drawer::with_options(drawer_options);
        if ctx.state.lsp_features().legacy_vue2 {
            drawer = drawer.with_legacy_vue2();
        } else if ctx.state.options_api_enabled() {
            drawer = drawer.with_options_api();
        }

        if let Some(ref script) = descriptor.script {
            drawer.analyze_script_plain(&script.content);
        }
        if let Some(ref script_setup) = descriptor.script_setup {
            drawer.analyze_script_setup(&script_setup.content);
        }

        // Analyze template if present
        if let Some(ref template) = descriptor.template {
            let allocator = vize_carton::Allocator::new();
            let (root, _) = vize_armature::parse(&allocator, &template.content);
            drawer.analyze_template(&root);
        }

        let summary = drawer.finish();

        // Look up the binding in the analysis summary
        let binding_type = summary.get_binding_type(word)?;

        // Try to infer a more specific type from the script content
        let inferred_type = script_content
            .and_then(|content| Self::infer_type_from_script(content, word, binding_type))
            .unwrap_or_else(|| Self::binding_type_to_ts_display(binding_type).to_string());

        // Format the hover content
        let kind_desc = Self::binding_type_to_description(binding_type);
        let source = if matches!(
            binding_type,
            BindingType::Data | BindingType::Options | BindingType::VueGlobal
        ) {
            "`<script>`"
        } else if descriptor.script_setup.is_some() {
            "`<script setup>`"
        } else {
            "`<script>`"
        };
        let resolved_from = if descriptor.script_setup.is_some()
            && !matches!(
                binding_type,
                BindingType::Data | BindingType::Options | BindingType::VueGlobal
            ) {
            "The binding is resolved from `<script setup>` analysis."
        } else {
            "The binding is resolved from `<script>` analysis."
        };

        #[allow(clippy::disallowed_macros)]
        let signature = format!("{word}: {inferred_type}");

        Some(
            HoverBuilder::new()
                .title(word)
                .meta("Template binding from script")
                .code("typescript", &signature)
                .description(kind_desc)
                .section("Source", source)
                .bullets(
                    "Template behavior",
                    &[
                        "Ref values are automatically unwrapped in templates.",
                        resolved_from,
                    ],
                )
                .build(),
        )
    }

    /// Get hover for Vue directives.
    pub(super) fn hover_directive(word: &str) -> Option<Hover> {
        let (title, description) = match word {
            "v-if" => (
                "v-if",
                "Conditionally render the element based on the truthy-ness of the expression value.",
            ),
            "v-else-if" => (
                "v-else-if",
                "Denote the \"else if block\" for `v-if`. Can be chained.",
            ),
            "v-else" => (
                "v-else",
                "Denote the \"else block\" for `v-if` or `v-if`/`v-else-if` chain.",
            ),
            "v-for" => (
                "v-for",
                "Render the element or template block multiple times based on the source data.",
            ),
            "v-on" | "@" => (
                "v-on",
                "Attach an event listener to the element. The event type is denoted by the argument.",
            ),
            "v-bind" | ":" => (
                "v-bind",
                "Dynamically bind one or more attributes, or a component prop to an expression.",
            ),
            "v-model" => (
                "v-model",
                "Create a two-way binding on a form input element or a component.",
            ),
            "v-slot" | "#" => (
                "v-slot",
                "Denote named slots or scoped slots that expect to receive props.",
            ),
            "v-pre" => (
                "v-pre",
                "Skip compilation for this element and all its children.",
            ),
            "v-once" => (
                "v-once",
                "Render the element and component once only, and skip future updates.",
            ),
            "v-memo" => (
                "v-memo",
                "Memoize a sub-tree of the template. Can be used on both elements and components.",
            ),
            "v-cloak" => (
                "v-cloak",
                "Used to hide un-compiled template until it is ready.",
            ),
            "v-show" => (
                "v-show",
                "Toggle the element's visibility based on the truthy-ness of the expression value.",
            ),
            "v-text" => ("v-text", "Update the element's text content."),
            "v-html" => ("v-html", "Update the element's innerHTML."),
            _ => return None,
        };

        Some(
            HoverBuilder::new()
                .title(title)
                .meta("Vue template directive")
                .description(description)
                .bullets(
                    "Usage",
                    &[
                        "Use inside `<template>` markup.",
                        "Directive expressions are evaluated in component scope.",
                    ],
                )
                .docs(
                    "Vue Built-in Directives",
                    "https://vuejs.org/api/built-in-directives.html",
                )
                .build(),
        )
    }
}

fn template_attribute_name_at_offset(content: &str, offset: usize) -> Option<&str> {
    let cursor = offset.min(content.len());
    let tag_start = content[..cursor].rfind('<')?;
    let bytes = content.as_bytes();
    if matches!(bytes.get(tag_start + 1), Some(b'/' | b'!' | b'?')) {
        return None;
    }

    let tag_end = find_open_tag_end(content, tag_start)?;
    if cursor > tag_end {
        return None;
    }

    let mut pos = tag_start + 1;
    while pos < tag_end {
        let byte = bytes[pos];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            pos += 1;
        } else {
            break;
        }
    }

    while pos < tag_end {
        while pos < tag_end && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= tag_end || matches!(bytes[pos], b'/' | b'>') {
            break;
        }

        let attr_start = pos;
        while pos < tag_end
            && !bytes[pos].is_ascii_whitespace()
            && !matches!(bytes[pos], b'=' | b'/' | b'>')
        {
            pos += 1;
        }
        let attr_end = pos;
        if attr_start == attr_end {
            return None;
        }

        if cursor >= attr_start && cursor <= attr_end {
            return Some(&content[attr_start..attr_end]);
        }

        while pos < tag_end && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos < tag_end && bytes[pos] == b'=' {
            pos += 1;
            while pos < tag_end && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos < tag_end && matches!(bytes[pos], b'"' | b'\'') {
                let quote = bytes[pos];
                pos += 1;
                while pos < tag_end && bytes[pos] != quote {
                    pos += 1;
                }
                if pos < tag_end {
                    pos += 1;
                }
            } else {
                while pos < tag_end && !bytes[pos].is_ascii_whitespace() && bytes[pos] != b'>' {
                    pos += 1;
                }
            }
        }
    }

    None
}

fn find_open_tag_end(content: &str, tag_start: usize) -> Option<usize> {
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
