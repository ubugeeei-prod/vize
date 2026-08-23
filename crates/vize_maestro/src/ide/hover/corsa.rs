//! Corsa integration for hover.
//!
//! Provides offset conversion between SFC and virtual TypeScript documents,
//! and conversion of Corsa hover responses to LSP hover format.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use std::sync::Arc;

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};
use vize_canon::{CorsaBridge, LspHover, LspHoverContents, LspMarkedString};

use super::HoverService;
use crate::ide::IdeContext;
use crate::virtual_code::ArtVariantInfo;

mod markdown;

impl HoverService {
    /// Convert SFC offset to virtual TS template offset.
    pub(crate) fn sfc_to_virtual_ts_offset(
        ctx: &IdeContext<'_>,
        sfc_offset: usize,
    ) -> Option<usize> {
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let template = virtual_docs.template.as_ref()?;

        if crate::utils::is_standalone_html_path(ctx.uri.path()) {
            return template
                .source_map
                .to_generated(sfc_offset as u32)
                .map(|o| o as usize)
                .or(Some(sfc_offset));
        }

        // Get template block start offset in SFC
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;
        let template_block = descriptor.template.as_ref()?;
        let template_start = template_block.loc.start;

        // Check if offset is within template
        if sfc_offset < template_start || sfc_offset > template_block.loc.end {
            return None;
        }

        // Calculate relative offset
        let relative_offset = sfc_offset - template_start;

        // Use source map to convert offset
        template
            .source_map
            .to_generated(relative_offset as u32)
            .map(|o| o as usize)
            .or(Some(relative_offset))
    }

    /// Convert SFC offset to virtual TS script offset.
    pub(crate) fn sfc_to_virtual_ts_script_offset(
        ctx: &IdeContext<'_>,
        sfc_offset: usize,
    ) -> Option<usize> {
        let virtual_docs = ctx.virtual_docs.as_ref()?;

        if ctx.uri.path().ends_with(".art.vue")
            && let Some(ref script_setup_doc) = virtual_docs.script_setup
            && let Some(offset) = script_setup_doc.source_map.to_generated(sfc_offset as u32)
        {
            return Some(offset as usize);
        }

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

        // Try script setup first
        if let Some(ref script_setup) = descriptor.script_setup
            && sfc_offset >= script_setup.loc.start
            && sfc_offset <= script_setup.loc.end
        {
            let relative_offset = sfc_offset - script_setup.loc.start;
            if let Some(ref script_setup_doc) = virtual_docs.script_setup {
                return script_setup_doc
                    .source_map
                    .to_generated(relative_offset as u32)
                    .map(|o| o as usize)
                    .or(Some(relative_offset));
            }
            return Some(relative_offset);
        }

        // Try regular script
        if let Some(ref script) = descriptor.script
            && sfc_offset >= script.loc.start
            && sfc_offset <= script.loc.end
        {
            let relative_offset = sfc_offset - script.loc.start;
            if let Some(ref script_doc) = virtual_docs.script {
                return script_doc
                    .source_map
                    .to_generated(relative_offset as u32)
                    .map(|o| o as usize)
                    .or(Some(relative_offset));
            }
            return Some(relative_offset);
        }

        None
    }

    /// Get hover for template context with Corsa support.
    pub(super) async fn hover_template_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset)
            && let Some(mut hover) = Self::hover_component_tag(ctx)
        {
            if hover.range.is_none() {
                hover.range = authored_hover_token_range(ctx);
            }
            return Some(hover);
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset)
            && let Some(mut hover) = super::component_prop::hover_attribute(ctx)
        {
            if hover.range.is_none() {
                hover.range = authored_hover_token_range(ctx);
            }
            return Some(hover);
        }

        if !word.is_empty()
            && let Some(hover) = Self::hover_directive(&word)
        {
            return Some(hover);
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset)
            && let Some(hover) =
                Self::hover_html_attribute_with_corsa(ctx, corsa_bridge.as_ref()).await
        {
            return Some(hover);
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset)
            && let Some(hover) = Self::hover_html_tag_with_corsa(ctx, corsa_bridge.as_ref()).await
        {
            return Some(hover);
        }

        // petite-vue standalone HTML `v-scope` bindings have no virtual TS
        // declaration for Corsa to resolve; surface them from the scope chain.
        if !word.is_empty()
            && let Some(hover) = Self::hover_petite_vue_scope_binding(ctx, &word)
        {
            return Some(hover);
        }

        if let Some(bridge) = corsa_bridge.as_ref()
            && bridge.is_initialized()
            && let Some(doc) =
                crate::ide::corsa_support::open_canonical_virtual_document(ctx, bridge).await
            && let Some((line, character)) =
                crate::ide::corsa_support::canonical_source_offset_to_position(&doc, ctx.offset)
            && let Ok(Some(hover)) = bridge.hover(&doc.request_uri, line, character).await
        {
            let mapped_range = hover.range.as_ref().and_then(|range| {
                crate::ide::corsa_support::map_canonical_lsp_range(ctx, &doc, range)
            });
            let mut converted = Self::convert_lsp_hover(hover);
            converted.range = mapped_range.or_else(|| authored_hover_token_range(ctx));
            super::declaration_keyword::align_hover(ctx, &word, &mut converted);
            return Some(converted);
        }

        if word.is_empty() {
            return None;
        }

        Self::hover_template(ctx).map(|mut hover| {
            if hover.range.is_none() {
                hover.range = authored_hover_token_range(ctx);
            }
            hover
        })
    }

    /// Get hover for an art variant template with Corsa.
    ///
    /// Maps the art variant offset to the virtual TS offset and requests hover from Corsa.
    pub(super) async fn hover_art_variant_with_corsa(
        ctx: &IdeContext<'_>,
        info: &ArtVariantInfo,
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

        // Try to get type information from Corsa via virtual TypeScript.
        // Typed art documents share absolute SFC offsets across script and template
        // mappings, but a variant whose expression has no generated counterpart must
        // still answer from the authored template rather than returning nothing.
        if let Some(bridge) = corsa_bridge
            && let Some(ref virtual_docs) = ctx.virtual_docs
            && let Some(template) = virtual_docs.art_template(info.variant_index)
            && let Some(vts_offset) = template.source_map.to_generated(ctx.offset as u32)
        {
            let vts_offset = vts_offset as usize;

            let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);

            // Open/update virtual document
            if bridge.is_initialized() {
                let vdoc_uri = crate::ide::corsa_support::art_template_request_path(
                    ctx.uri,
                    info.variant_index,
                );
                let Ok(uri) = bridge
                    .open_or_update_virtual_document(&vdoc_uri, &template.content)
                    .await
                else {
                    return Self::hover_template(ctx);
                };

                // Request hover from Corsa.
                if let Ok(Some(hover)) = bridge.hover(&uri, line, character).await {
                    let mapped_range = hover.range.as_ref().and_then(|range| {
                        crate::ide::corsa_support::map_virtual_range(
                            ctx,
                            template,
                            &Range {
                                start: tower_lsp::lsp_types::Position {
                                    line: range.start.line,
                                    character: range.start.character,
                                },
                                end: tower_lsp::lsp_types::Position {
                                    line: range.end.line,
                                    character: range.end.character,
                                },
                            },
                        )
                    });
                    let mut converted = Self::convert_lsp_hover(hover);
                    converted.range = mapped_range;
                    return Some(converted);
                }
            }
        }

        // Fall back to template hover (croquis analysis)
        Self::hover_template(ctx)
    }

    /// Convert a Corsa hover payload to tower-lsp Hover.
    pub(in crate::ide) fn convert_lsp_hover(lsp_hover: LspHover) -> Hover {
        let value = match lsp_hover.contents {
            LspHoverContents::Markup(markup) => {
                if markup.kind == "markdown" {
                    markup.value
                } else {
                    // Wrap plaintext TypeScript type info in a code block for better rendering
                    Self::wrap_type_info_in_codeblock(&markup.value)
                }
            }
            LspHoverContents::String(s) => {
                // Wrap plaintext in a TypeScript code block
                Self::wrap_type_info_in_codeblock(&s)
            }
            LspHoverContents::Array(items) => items
                .into_iter()
                .map(|item| match item {
                    LspMarkedString::String(s) => Self::wrap_type_info_in_codeblock(&s),
                    LspMarkedString::LanguageString { language, value } => {
                        #[allow(clippy::disallowed_macros)]
                        {
                            format!("```{}\n{}\n```", language, value)
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join("\n\n"),
        };

        let contents = HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: Self::decorate_corsa_hover_markdown(&value),
        });

        let range = lsp_hover.range.map(|r| Range {
            start: tower_lsp::lsp_types::Position {
                line: r.start.line,
                character: r.start.character,
            },
            end: tower_lsp::lsp_types::Position {
                line: r.end.line,
                character: r.end.character,
            },
        });

        Hover { contents, range }
    }
}

fn authored_hover_token_range(ctx: &IdeContext<'_>) -> Option<Range> {
    if let Some((start, end)) = super::v_model::argument_token_span(&ctx.content, ctx.offset) {
        let (start_line, start_character) = crate::ide::offset_to_position(&ctx.content, start);
        let (end_line, end_character) = crate::ide::offset_to_position(&ctx.content, end);
        return Some(Range::new(
            Position::new(start_line, start_character),
            Position::new(end_line, end_character),
        ));
    }

    let (start, end) =
        crate::ide::token_span_at_offset(&ctx.content, ctx.offset, HoverService::is_word_char)?;
    let (start_line, start_character) = crate::ide::offset_to_position(&ctx.content, start);
    let (end_line, end_character) = crate::ide::offset_to_position(&ctx.content, end);
    Some(Range::new(
        Position::new(start_line, start_character),
        Position::new(end_line, end_character),
    ))
}
