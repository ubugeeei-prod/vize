//! Tsgo integration for hover.
//!
//! Provides offset conversion between SFC and virtual TypeScript documents,
//! and conversion of tsgo hover responses to LSP hover format.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use std::sync::Arc;

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Range};
use vize_canon::{LspHover, LspHoverContents, LspMarkedString, TsgoBridge};

use super::HoverService;
use crate::ide::IdeContext;
use crate::virtual_code::ArtVariantInfo;

impl HoverService {
    /// Convert SFC offset to virtual TS template offset.
    pub(crate) fn sfc_to_virtual_ts_offset(
        ctx: &IdeContext<'_>,
        sfc_offset: usize,
    ) -> Option<usize> {
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let template = virtual_docs.template.as_ref()?;

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

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

        // Try script setup first
        if let Some(ref script_setup) = descriptor.script_setup {
            if sfc_offset >= script_setup.loc.start && sfc_offset <= script_setup.loc.end {
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
        }

        // Try regular script
        if let Some(ref script) = descriptor.script {
            if sfc_offset >= script.loc.start && sfc_offset <= script.loc.end {
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
        }

        None
    }

    /// Get hover for an art variant template with tsgo.
    ///
    /// Maps the art variant offset to the virtual TS offset and requests hover from tsgo.
    pub(super) async fn hover_art_variant_with_tsgo(
        ctx: &IdeContext<'_>,
        info: &ArtVariantInfo,
        tsgo_bridge: Option<Arc<TsgoBridge>>,
    ) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue directives first (these don't need tsgo)
        if let Some(hover) = Self::hover_directive(&word) {
            return Some(hover);
        }

        // Try to get type information from tsgo via virtual TypeScript
        if let Some(bridge) = tsgo_bridge {
            if let Some(ref virtual_docs) = ctx.virtual_docs {
                if let Some(ref template) = virtual_docs.template {
                    // Convert the art variant relative offset through the template source map
                    let relative_offset = info.relative_offset as u32;
                    let vts_offset = template
                        .source_map
                        .to_generated(relative_offset)
                        .map(|o| o as usize)
                        .unwrap_or(relative_offset as usize);

                    let (line, character) =
                        crate::ide::offset_to_position(&template.content, vts_offset);
                    #[allow(clippy::disallowed_macros)]
                    let uri = format!("vize-virtual://{}.template.ts", ctx.uri.path());

                    // Open/update virtual document
                    if bridge.is_initialized() {
                        #[allow(clippy::disallowed_macros)]
                        let vdoc_uri = format!("{}.template.ts", ctx.uri.path());
                        let _ = bridge
                            .open_or_update_virtual_document(&vdoc_uri, &template.content)
                            .await;

                        // Request hover from tsgo
                        if let Ok(Some(hover)) = bridge.hover(&uri, line, character).await {
                            return Some(Self::convert_lsp_hover(hover));
                        }
                    }
                }
            }
        }

        // Fall back to template hover (croquis analysis)
        Self::hover_template(ctx)
    }

    /// Convert tsgo LspHover to tower-lsp Hover.
    pub(super) fn convert_lsp_hover(lsp_hover: LspHover) -> Hover {
        let contents = match lsp_hover.contents {
            LspHoverContents::Markup(markup) => {
                let value = if markup.kind == "markdown" {
                    markup.value
                } else {
                    // Wrap plaintext TypeScript type info in a code block for better rendering
                    Self::wrap_type_info_in_codeblock(&markup.value)
                };
                HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                })
            }
            LspHoverContents::String(s) => {
                // Wrap plaintext in a TypeScript code block
                HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: Self::wrap_type_info_in_codeblock(&s),
                })
            }
            LspHoverContents::Array(items) => {
                let value = items
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
                    .join("\n\n");
                HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                })
            }
        };

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

    /// Wrap TypeScript type information in a code block for proper markdown rendering.
    pub(super) fn wrap_type_info_in_codeblock(text: &str) -> String {
        let text = text.trim();
        // If already wrapped in code block, return as-is
        if text.starts_with("```") {
            return text.to_string();
        }
        // Check if this looks like TypeScript type info
        // Common patterns: (const), (let), (var), (function), (method), (property), type, interface, etc.
        let looks_like_type_info = text.starts_with('(')
            || text.starts_with("type ")
            || text.starts_with("interface ")
            || text.starts_with("class ")
            || text.starts_with("enum ")
            || text.starts_with("function ")
            || text.starts_with("const ")
            || text.starts_with("let ")
            || text.starts_with("var ")
            || text.starts_with("import ")
            || text.contains(": ")
            || text.contains("=>")
            || text.contains(" | ")
            || text.contains(" & ");

        if looks_like_type_info {
            #[allow(clippy::disallowed_macros)]
            {
                format!("```typescript\n{}\n```", text)
            }
        } else {
            text.to_string()
        }
    }
}
