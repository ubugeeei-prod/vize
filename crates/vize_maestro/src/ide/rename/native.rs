//! Native rename dispatch shared by the public service.

use std::sync::Arc;
use tower_lsp::lsp_types::WorkspaceEdit;
use vize_canon::CorsaBridge;

use super::{ArtCursorPosition, BlockType, IdeContext, RenameService, canonical, corsa};

impl RenameService {
    /// Perform rename using Corsa when possible, with synchronous fallback.
    #[cfg(feature = "native")]
    pub async fn rename_with_corsa(
        ctx: &IdeContext<'_>,
        new_name: &str,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<WorkspaceEdit> {
        if let canonical::Answer::Available(edit) =
            canonical::rename(ctx, new_name, corsa_bridge.as_deref()).await
        {
            return corsa::merge_missing_authored_rename(ctx, edit, Self::rename(ctx, new_name));
        }
        let corsa_result = match ctx.block_type? {
            BlockType::Template => {
                Self::rename_template_with_corsa(ctx, new_name, corsa_bridge.as_deref()).await
            }
            BlockType::Script | BlockType::ScriptSetup => {
                Self::rename_script_with_corsa(
                    ctx,
                    new_name,
                    matches!(ctx.block_type, Some(BlockType::ScriptSetup)),
                    corsa_bridge.as_deref(),
                )
                .await
            }
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::rename_art_variant_with_corsa(ctx, info, new_name, corsa_bridge.as_deref())
                    .await
            }
            BlockType::Style(_) | BlockType::Art(_) => None,
        };

        // Corsa only renames the virtual document the request opened, so the
        // authored edits carry the other blocks of this SFC.
        corsa::merge_authored_rename(ctx, corsa_result, Self::rename(ctx, new_name))
    }
}
