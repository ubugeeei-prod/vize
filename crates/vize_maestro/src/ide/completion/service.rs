//! Completion service entry point and Corsa integration.
//!
//! Provides the main `complete` and `complete_with_corsa` methods
//! that dispatch to block-specific handlers.
#![allow(clippy::disallowed_types)]

#[cfg(feature = "native")]
use std::sync::Arc;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Documentation, InsertTextFormat,
    MarkupContent, MarkupKind,
};

#[cfg(feature = "native")]
use vize_canon::{CorsaBridge, LspCompletionItem, LspDocumentation};

use super::{is_inside_html_comment, script, style, template};
#[cfg(feature = "native")]
use crate::ide::corsa_support;
use crate::ide::IdeContext;
use crate::virtual_code::{ArtCursorPosition, BlockType};

impl super::CompletionService {
    /// Get completions for the given context.
    pub fn complete(ctx: &IdeContext) -> Option<CompletionResponse> {
        // Art file: route by cursor position within art structure
        if ctx.uri.path().ends_with(".art.vue") {
            return match ctx.block_type {
                Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_))) => {
                    // Inside variant template: provide template completions
                    let items = template::complete_template(ctx);
                    if items.is_empty() {
                        template::complete_art(ctx)
                    } else {
                        Some(CompletionResponse::Array(items))
                    }
                }
                Some(BlockType::ScriptSetup) => {
                    let items = script::complete_script(ctx, true);
                    if items.is_empty() {
                        None
                    } else {
                        Some(CompletionResponse::Array(items))
                    }
                }
                Some(BlockType::Script) => {
                    let items = script::complete_script(ctx, false);
                    if items.is_empty() {
                        None
                    } else {
                        Some(CompletionResponse::Array(items))
                    }
                }
                _ => template::complete_art(ctx),
            };
        }

        // Check if cursor is inside <art> block in a regular .vue file
        if matches!(ctx.block_type, Some(BlockType::Art(_))) {
            return template::complete_inline_art(ctx);
        }

        let items = match ctx.block_type? {
            BlockType::Template => template::complete_template(ctx),
            BlockType::Script => script::complete_script(ctx, false),
            BlockType::ScriptSetup => script::complete_script(ctx, true),
            BlockType::Style(index) => style::complete_style(ctx, index),
            BlockType::Art(_) => unreachable!(), // handled above
        };

        if items.is_empty() {
            None
        } else {
            Some(CompletionResponse::Array(items))
        }
    }

    /// Get completions with Corsa support (async version).
    #[cfg(feature = "native")]
    pub async fn complete_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<CompletionResponse> {
        // Art file: route by cursor position within art structure
        if ctx.uri.path().ends_with(".art.vue") {
            return match ctx.block_type {
                Some(BlockType::Art(ArtCursorPosition::VariantTemplate(ref info))) => {
                    // Try Corsa template completion for variant template.
                    if let Some(ref bridge) = corsa_bridge {
                        let items = Self::complete_art_variant_with_corsa(ctx, info, bridge).await;
                        if !items.is_empty() {
                            let mut all = items;
                            all.extend(template::directive_completions());
                            return Some(CompletionResponse::Array(all));
                        }
                    }
                    // Fallback to structural completions
                    Self::complete(ctx)
                }
                Some(BlockType::ScriptSetup) => {
                    // Script setup in art file: use normal script completion with Corsa.
                    if let Some(ref bridge) = corsa_bridge {
                        let items = Self::complete_script_with_corsa(ctx, true, bridge).await;
                        if !items.is_empty() {
                            let mut all = items;
                            let mut v = script::composition_api_completions();
                            v.extend(script::macro_completions());
                            all.extend(v);
                            return Some(CompletionResponse::Array(all));
                        }
                    }
                    Self::complete(ctx)
                }
                Some(BlockType::Script) => {
                    if let Some(ref bridge) = corsa_bridge {
                        let items = Self::complete_script_with_corsa(ctx, false, bridge).await;
                        if !items.is_empty() {
                            let mut all = items;
                            all.extend(script::composition_api_completions());
                            return Some(CompletionResponse::Array(all));
                        }
                    }
                    Self::complete(ctx)
                }
                _ => Self::complete(ctx),
            };
        }

        // Check if cursor is inside <art> block in a regular .vue file
        if matches!(ctx.block_type, Some(BlockType::Art(_))) {
            return template::complete_inline_art(ctx);
        }

        let block_type = ctx.block_type?;

        // If in template and cursor is inside an HTML comment, return directive completions only
        if matches!(block_type, BlockType::Template)
            && is_inside_html_comment(&ctx.content, ctx.offset)
        {
            let items = template::vize_directive_completions();
            return if items.is_empty() {
                None
            } else {
                Some(CompletionResponse::Array(items))
            };
        }

        // Try Corsa completion first.
        if let Some(bridge) = corsa_bridge {
            let corsa_items = match block_type {
                BlockType::Template => Self::complete_template_with_corsa(ctx, &bridge).await,
                BlockType::Script => Self::complete_script_with_corsa(ctx, false, &bridge).await,
                BlockType::ScriptSetup => {
                    Self::complete_script_with_corsa(ctx, true, &bridge).await
                }
                BlockType::Style(_) => vec![],
                BlockType::Art(_) => vec![],
            };

            if !corsa_items.is_empty() {
                let mut items = corsa_items;
                items.extend(match block_type {
                    BlockType::Template => template::directive_completions(),
                    BlockType::Script => script::composition_api_completions(),
                    BlockType::ScriptSetup => {
                        let mut v = script::composition_api_completions();
                        v.extend(script::macro_completions());
                        v
                    }
                    BlockType::Style(_) => style::vue_css_completions(),
                    BlockType::Art(_) => vec![],
                });

                return Some(CompletionResponse::Array(items));
            }
        }

        // Fall back to synchronous completions
        Self::complete(ctx)
    }

    /// Get completions for an art variant template with Corsa.
    #[cfg(feature = "native")]
    async fn complete_art_variant_with_corsa(
        ctx: &IdeContext<'_>,
        info: &crate::virtual_code::ArtVariantInfo,
        bridge: &CorsaBridge,
    ) -> Vec<CompletionItem> {
        if let Some(ref virtual_docs) = ctx.virtual_docs {
            if let Some(ref tmpl) = virtual_docs.template {
                // Convert the art variant relative offset through the template source map
                let relative_offset = info.relative_offset as u32;
                let vts_offset = tmpl
                    .source_map
                    .to_generated(relative_offset)
                    .map(|o| o as usize)
                    .unwrap_or(relative_offset as usize);

                let (line, character) = crate::ide::offset_to_position(&tmpl.content, vts_offset);

                if bridge.is_initialized() {
                    let request_path = corsa_support::template_request_path(ctx.uri);
                    let Ok(uri) = bridge
                        .open_or_update_virtual_document(&request_path, &tmpl.content)
                        .await
                    else {
                        return vec![];
                    };

                    if let Ok(items) = bridge.completion(&uri, line, character).await {
                        return items
                            .into_iter()
                            .map(Self::convert_lsp_completion)
                            .collect();
                    }
                }
            }
        }

        vec![]
    }

    /// Get completions for template with Corsa.
    #[cfg(feature = "native")]
    async fn complete_template_with_corsa(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Vec<CompletionItem> {
        if let Some(ref virtual_docs) = ctx.virtual_docs {
            if let Some(ref tmpl) = virtual_docs.template {
                if let Some(vts_offset) =
                    crate::ide::hover::HoverService::sfc_to_virtual_ts_offset(ctx, ctx.offset)
                {
                    let (line, character) =
                        crate::ide::offset_to_position(&tmpl.content, vts_offset);

                    if bridge.is_initialized() {
                        let request_path = corsa_support::template_request_path(ctx.uri);
                        let Ok(uri) = bridge
                            .open_or_update_virtual_document(&request_path, &tmpl.content)
                            .await
                        else {
                            return vec![];
                        };

                        if let Ok(items) = bridge.completion(&uri, line, character).await {
                            return items
                                .into_iter()
                                .map(Self::convert_lsp_completion)
                                .collect();
                        }
                    }
                }
            }
        }

        vec![]
    }

    /// Get completions for script with Corsa.
    #[cfg(feature = "native")]
    async fn complete_script_with_corsa(
        ctx: &IdeContext<'_>,
        is_setup: bool,
        bridge: &CorsaBridge,
    ) -> Vec<CompletionItem> {
        if let Some(ref virtual_docs) = ctx.virtual_docs {
            let script_doc = if is_setup {
                virtual_docs.script_setup.as_ref()
            } else {
                virtual_docs.script.as_ref()
            };

            if let Some(s) = script_doc {
                if let Some(vts_offset) =
                    crate::ide::hover::HoverService::sfc_to_virtual_ts_script_offset(
                        ctx, ctx.offset,
                    )
                {
                    let (line, character) = crate::ide::offset_to_position(&s.content, vts_offset);

                    if bridge.is_initialized() {
                        let request_path = corsa_support::script_request_path(ctx.uri, is_setup);
                        let Ok(uri) = bridge
                            .open_or_update_virtual_document(&request_path, &s.content)
                            .await
                        else {
                            return vec![];
                        };

                        if let Ok(items) = bridge.completion(&uri, line, character).await {
                            return items
                                .into_iter()
                                .map(Self::convert_lsp_completion)
                                .collect();
                        }
                    }
                }
            }
        }

        vec![]
    }

    /// Convert a Corsa completion item to tower-lsp CompletionItem.
    #[cfg(feature = "native")]
    fn convert_lsp_completion(item: LspCompletionItem) -> CompletionItem {
        CompletionItem {
            label: item.label,
            kind: item.kind.map(Self::convert_completion_kind),
            detail: item.detail,
            documentation: item.documentation.map(|doc| match doc {
                LspDocumentation::String(s) => Documentation::String(s),
                LspDocumentation::Markup(m) => Documentation::MarkupContent(MarkupContent {
                    kind: if m.kind == "markdown" {
                        MarkupKind::Markdown
                    } else {
                        MarkupKind::PlainText
                    },
                    value: m.value,
                }),
            }),
            insert_text: item.insert_text,
            insert_text_format: item.insert_text_format.map(|f| {
                if f == 2 {
                    InsertTextFormat::SNIPPET
                } else {
                    InsertTextFormat::PLAIN_TEXT
                }
            }),
            filter_text: item.filter_text,
            sort_text: item.sort_text,
            ..Default::default()
        }
    }

    /// Convert LSP completion item kind number to CompletionItemKind.
    #[cfg(feature = "native")]
    fn convert_completion_kind(kind: u32) -> CompletionItemKind {
        match kind {
            1 => CompletionItemKind::TEXT,
            2 => CompletionItemKind::METHOD,
            3 => CompletionItemKind::FUNCTION,
            4 => CompletionItemKind::CONSTRUCTOR,
            5 => CompletionItemKind::FIELD,
            6 => CompletionItemKind::VARIABLE,
            7 => CompletionItemKind::CLASS,
            8 => CompletionItemKind::INTERFACE,
            9 => CompletionItemKind::MODULE,
            10 => CompletionItemKind::PROPERTY,
            11 => CompletionItemKind::UNIT,
            12 => CompletionItemKind::VALUE,
            13 => CompletionItemKind::ENUM,
            14 => CompletionItemKind::KEYWORD,
            15 => CompletionItemKind::SNIPPET,
            16 => CompletionItemKind::COLOR,
            17 => CompletionItemKind::FILE,
            18 => CompletionItemKind::REFERENCE,
            19 => CompletionItemKind::FOLDER,
            20 => CompletionItemKind::ENUM_MEMBER,
            21 => CompletionItemKind::CONSTANT,
            22 => CompletionItemKind::STRUCT,
            23 => CompletionItemKind::EVENT,
            24 => CompletionItemKind::OPERATOR,
            25 => CompletionItemKind::TYPE_PARAMETER,
            _ => CompletionItemKind::TEXT,
        }
    }
}
