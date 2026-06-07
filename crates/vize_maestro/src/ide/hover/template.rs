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
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_relief::BindingType;

#[cfg(feature = "native")]
use std::sync::Arc;

#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

use super::{HoverBuilder, HoverService};
use crate::ide::IdeContext;

impl HoverService {
    /// Get hover for template context.
    pub(super) fn hover_template(ctx: &IdeContext) -> Option<Hover> {
        // Try to find what's under the cursor
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue directives
        if let Some(hover) = Self::hover_directive(&word) {
            return Some(hover);
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
            return None;
        }

        // Try to get TypeScript type information from croquis analysis
        if let Some(hover) = Self::hover_ts_binding(ctx, &word) {
            return Some(hover);
        }

        // Try to get type information from vize_canon
        if let Some(type_info) = crate::ide::TypeService::get_type_at(ctx) {
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

        // Check for template bindings from script setup
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

        // Default: show it's a template expression
        Some(
            HoverBuilder::new()
                .title(&word)
                .meta("Template expression")
                .description("Expression evaluated against the component template scope.")
                .build(),
        )
    }

    /// Get hover for template context with Corsa support.
    #[cfg(feature = "native")]
    pub(super) async fn hover_template_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue directives first; these do not need Corsa.
        if let Some(hover) = Self::hover_directive(&word) {
            return Some(hover);
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
            return None;
        }

        // Try to get type information from Corsa via virtual TypeScript.
        if let Some(bridge) = corsa_bridge
            && let Some(ref virtual_docs) = ctx.virtual_docs
            && let Some(ref template) = virtual_docs.template
        {
            // Calculate position in virtual TS
            if let Some(vts_offset) = Self::sfc_to_virtual_ts_offset(ctx, ctx.offset) {
                let (line, character) =
                    crate::ide::offset_to_position(&template.content, vts_offset);

                // Open/update virtual document
                if bridge.is_initialized() {
                    #[allow(clippy::disallowed_macros)]
                    let vdoc_uri = format!("{}.template.ts", ctx.uri.path());
                    let Ok(uri) = bridge
                        .open_or_update_virtual_document(&vdoc_uri, &template.content)
                        .await
                    else {
                        return Self::hover_template(ctx);
                    };

                    // Request hover from Corsa.
                    if let Ok(Some(hover)) = bridge.hover(&uri, line, character).await {
                        return Some(Self::convert_lsp_hover(hover));
                    }
                }
            }
        }

        // Fall back to croquis analysis
        Self::hover_template(ctx)
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

        // Create analyzer and analyze script
        let analyzer_options = AnalyzerOptions::full();
        let mut analyzer = Analyzer::with_options(analyzer_options);
        if ctx.state.lsp_features().legacy_vue2 {
            analyzer = analyzer.with_legacy_vue2();
        } else if ctx.state.options_api_enabled() {
            analyzer = analyzer.with_options_api();
        }

        if let Some(ref script) = descriptor.script {
            analyzer.analyze_script_plain(&script.content);
        }
        if let Some(ref script_setup) = descriptor.script_setup {
            analyzer.analyze_script_setup(&script_setup.content);
        }

        // Analyze template if present
        if let Some(ref template) = descriptor.template {
            let allocator = vize_carton::Bump::new();
            let (root, _) = vize_armature::parse(&allocator, &template.content);
            analyzer.analyze_template(&root);
        }

        let summary = analyzer.finish();

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
                .link(
                    "Vue Built-in Directives",
                    "https://vuejs.org/api/built-in-directives.html",
                )
                .build(),
        )
    }
}
