//! Inline `<art>` completion routing for regular Vue SFCs.

use tower_lsp::lsp_types::CompletionResponse;

#[cfg(feature = "native")]
use std::sync::Arc;
#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

use super::template;
use crate::ide::IdeContext;
use crate::virtual_code::{ArtCursorPosition, BlockType};

pub(super) fn complete(ctx: &IdeContext) -> Option<CompletionResponse> {
    let Some(BlockType::Art(ref art_position)) = ctx.block_type else {
        return None;
    };

    match art_position {
        ArtCursorPosition::VariantTemplate(_) => {
            let mut items = template::complete_template(ctx);
            if let Some(CompletionResponse::Array(inline_items)) =
                template::complete_inline_art(ctx)
            {
                items.extend(inline_items);
            }
            (!items.is_empty()).then_some(CompletionResponse::Array(items))
        }
        _ => template::complete_inline_art(ctx),
    }
}

#[cfg(feature = "native")]
pub(super) async fn complete_with_corsa(
    ctx: &IdeContext<'_>,
    corsa_bridge: Option<Arc<CorsaBridge>>,
) -> Option<CompletionResponse> {
    let Some(BlockType::Art(ref art_position)) = ctx.block_type else {
        return None;
    };

    match art_position {
        ArtCursorPosition::VariantTemplate(info) => {
            if let Some(ref bridge) = corsa_bridge {
                let items =
                    super::CompletionService::complete_art_variant_with_corsa(ctx, info, bridge)
                        .await;
                if !items.is_empty() {
                    let mut all = items;
                    all.extend(template::complete_template(ctx));
                    return Some(CompletionResponse::Array(all));
                }
            }
            super::CompletionService::complete(ctx)
        }
        _ => template::complete_inline_art(ctx),
    }
}
