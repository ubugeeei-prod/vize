//! Completion service entry point and Corsa integration.
//!
//! Dispatches `complete` and `complete_with_corsa` to block-specific handlers.
#![allow(clippy::disallowed_types)]

#[cfg(feature = "native")]
use std::sync::Arc;

use tower_lsp::lsp_types::CompletionResponse;
#[cfg(feature = "native")]
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent, MarkupKind,
};

#[cfg(feature = "native")]
use vize_canon::{CorsaBridge, LspCompletionItem, LspDocumentation};

#[cfg(feature = "native")]
use super::service_corsa_template;
use super::{script, service_inline_art, style, template};
#[cfg(feature = "native")]
use crate::ide::corsa_support;
use crate::ide::{IdeContext, ecosystem};
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

        if matches!(ctx.block_type, Some(BlockType::Art(_))) {
            return service_inline_art::complete(ctx);
        }

        if ctx.state.lsp_features().ecosystem {
            let ecosystem_items = ecosystem::completions(ctx);
            if !ecosystem_items.is_empty() {
                return Some(CompletionResponse::Array(ecosystem_items));
            }
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
                            all.extend(template::complete_template(ctx));
                            return Some(CompletionResponse::Array(all));
                        }
                    }
                    // Fallback to structural completions
                    Self::complete(ctx)
                }
                Some(BlockType::ScriptSetup) => {
                    if let Some(items) = crate::ide::musea::define_art_source_completions(ctx) {
                        return Some(CompletionResponse::Array(items));
                    }
                    // Script setup in art file: use normal script completion with Corsa.
                    if let Some(ref bridge) = corsa_bridge {
                        let items = Self::complete_script_with_corsa(ctx, true, bridge).await;
                        if !items.is_empty() {
                            let mut all = items;
                            all.extend(script::script_extras(ctx, true));
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
                            all.extend(script::script_extras(ctx, false));
                            return Some(CompletionResponse::Array(all));
                        }
                    }
                    Self::complete(ctx)
                }
                _ => Self::complete(ctx),
            };
        }

        if matches!(ctx.block_type, Some(BlockType::Art(_))) {
            return service_inline_art::complete_with_corsa(ctx, corsa_bridge).await;
        }

        if ctx.state.lsp_features().ecosystem {
            let ecosystem_items = ecosystem::completions(ctx);
            if !ecosystem_items.is_empty() {
                return Some(CompletionResponse::Array(ecosystem_items));
            }
        }

        let block_type = ctx.block_type?;

        // Inside HTML comments, only Vize directives are meaningful.
        if matches!(block_type, BlockType::Template)
            && super::is_inside_html_comment(&ctx.content, ctx.offset)
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
                BlockType::Template => service_corsa_template::complete(ctx, &bridge).await,
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
                    // Inside a template expression the checker's answer is
                    // the whole story; markup completions are noise (#3911).
                    BlockType::Template
                        if crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) =>
                    {
                        vec![]
                    }
                    BlockType::Template => template::corsa_template_completions(ctx),
                    BlockType::Script => script::script_extras(ctx, false),
                    BlockType::ScriptSetup => script::script_extras(ctx, true),
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
    pub(in crate::ide) async fn complete_art_variant_with_corsa(
        ctx: &IdeContext<'_>,
        info: &crate::virtual_code::ArtVariantInfo,
        bridge: &CorsaBridge,
    ) -> Vec<CompletionItem> {
        if let Some(ref virtual_docs) = ctx.virtual_docs
            && let Some(tmpl) = virtual_docs.art_template(info.variant_index)
        {
            // Typed art documents share absolute SFC offsets across script and template mappings.
            let Some(vts_offset) =
                corsa_support::completion_source_offset_to_generated(tmpl, ctx.offset as u32)
            else {
                return vec![];
            };

            let (line, character) = crate::ide::offset_to_position(&tmpl.content, vts_offset);

            if bridge.is_initialized() {
                let request_path =
                    corsa_support::art_template_request_path(ctx.uri, info.variant_index);
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

            if let Some(s) = script_doc
                && let Some(vts_offset) =
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
                        let mut items: Vec<_> = items
                            .into_iter()
                            .map(Self::convert_lsp_completion)
                            .collect();
                        improve_unknown_reactive_completions(ctx, is_setup, &mut items);
                        return items;
                    }
                }
            }
        }

        vec![]
    }

    /// Convert a Corsa completion item to tower-lsp CompletionItem.
    #[cfg(feature = "native")]
    pub(in crate::ide) fn convert_lsp_completion(item: LspCompletionItem) -> CompletionItem {
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

#[cfg(feature = "native")]
fn improve_unknown_reactive_completions(
    ctx: &IdeContext<'_>,
    is_setup: bool,
    items: &mut [CompletionItem],
) {
    if !items.iter().any(completion_has_unknown_reactive_type) {
        return;
    }

    let local_items = script::complete_script(ctx, is_setup);
    for item in items
        .iter_mut()
        .filter(|item| completion_has_unknown_reactive_type(item))
    {
        let Some(local) = local_items
            .iter()
            .find(|local| local.label == item.label && local_has_specific_reactive_type(local))
        else {
            continue;
        };
        item.detail = local.detail.clone();
        item.label_details = local.label_details.clone();
        item.documentation = local.documentation.clone();
    }
}

#[cfg(feature = "native")]
fn completion_has_unknown_reactive_type(item: &CompletionItem) -> bool {
    item.detail.as_deref().is_some_and(|detail| {
        detail.contains("Ref<unknown>") || detail.contains("ComputedRef<unknown>")
    })
}

#[cfg(feature = "native")]
fn local_has_specific_reactive_type(item: &CompletionItem) -> bool {
    item.detail.as_deref().is_some_and(|detail| {
        (detail.contains("Ref<") || detail.contains("ComputedRef<"))
            && !detail.contains("<unknown>")
    })
}
