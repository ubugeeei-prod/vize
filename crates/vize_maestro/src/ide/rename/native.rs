//! Native rename dispatch shared by the public service.

use std::sync::Arc;
use tower_lsp::lsp_types::{PrepareRenameResponse, WorkspaceEdit};
use vize_canon::CorsaBridge;

use super::{
    ArtCursorPosition, BlockType, IdeContext, NativeRename, RenameService, canonical, corsa,
};

#[cfg(test)]
mod prepare_tests;
#[cfg(test)]
mod private_tests;

impl RenameService {
    /// Perform rename using Corsa when possible, with synchronous fallback.
    #[cfg(feature = "native")]
    pub async fn rename_with_corsa(
        ctx: &IdeContext<'_>,
        new_name: &str,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<WorkspaceEdit> {
        if let Some(edit) = Self::private_setup_rename(ctx, new_name) {
            return Some(edit);
        }
        let (canonical_result, mut scope) =
            canonical::rename(ctx, new_name, corsa_bridge.as_deref()).await;
        match canonical_result {
            canonical::CanonicalRename::Refused => return None,
            canonical::CanonicalRename::Owned(edit) => {
                // A component argument owns the public property identity. Its
                // shorthand token also maps to a separate local value, which
                // the structural provider must not add to this transaction.
                if corsa::is_component_attribute_query(ctx) {
                    return corsa::merge_missing_authored_rename(ctx, Some(edit), None)
                        .filter(|edit| scope.admits_authored(edit));
                }
                return corsa::merge_missing_authored_rename(
                    ctx,
                    Some(edit),
                    Self::rename(ctx, new_name),
                )
                .filter(|edit| scope.admits_authored(edit));
            }
            canonical::CanonicalRename::Unavailable if corsa::is_component_attribute_query(ctx) => {
                return None;
            }
            canonical::CanonicalRename::Unavailable => {}
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

        let corsa_result = match corsa_result {
            Some(NativeRename::Refused) => return None,
            Some(NativeRename::Edit(edit)) => Some(edit),
            None => None,
        };

        // Corsa only renames the virtual document the request opened, so the
        // authored edits carry the other blocks of this SFC.
        corsa::merge_authored_rename(ctx, corsa_result, Self::rename(ctx, new_name))
            .filter(|edit| corsa::RenameScope::native(ctx).admits_authored(edit))
    }

    /// Return only a complete transaction for one positively proved private
    /// binding. Uncertainty is not refusal: the caller runs the old dispatcher.
    fn private_setup_rename(ctx: &IdeContext<'_>, new_name: &str) -> Option<WorkspaceEdit> {
        if ctx.uri.scheme() != "file"
            || !Self::is_valid_identifier(new_name)
            || Self::is_keyword(new_name)
        {
            return None;
        }
        let resident = Self::private_rename_resident(ctx)?;
        let (name, declaration) =
            crate::ide::ReferencesService::private_setup_rename_declaration(ctx)?;
        if name == new_name {
            return None;
        }
        let text_edits = crate::ide::references::structural::private_setup_rename(
            ctx,
            new_name,
            &name,
            declaration,
        )?;
        if text_edits.is_empty() {
            return None;
        }
        let mut spans = Vec::with_capacity(text_edits.len());
        for edit in &text_edits {
            let start = crate::ide::position_to_offset(
                &ctx.content,
                edit.range.start.line,
                edit.range.start.character,
            )?;
            let end = crate::ide::position_to_offset(
                &ctx.content,
                edit.range.end.line,
                edit.range.end.character,
            )?;
            if start >= end
                || crate::ide::offset_to_position(&ctx.content, start)
                    != (edit.range.start.line, edit.range.start.character)
                || crate::ide::offset_to_position(&ctx.content, end)
                    != (edit.range.end.line, edit.range.end.character)
                || ctx.content.get(start..end)? != name.as_str()
                || edit.new_text != new_name
            {
                return None;
            }
            spans.push((start, end));
        }
        spans.sort_unstable();
        if !spans.contains(&declaration)
            || spans.windows(2).any(|pair| {
                pair.first()
                    .zip(pair.get(1))
                    .is_none_or(|(left, right)| left.1 > right.0)
            })
        {
            return None;
        }
        let mut changes = std::collections::HashMap::new();
        changes.insert(ctx.uri.clone(), text_edits);
        let edit = WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        };
        if !corsa::RenameScope::native(ctx).admits_authored(&edit)
            || Self::private_rename_resident(ctx)? != resident
        {
            return None;
        }
        Some(edit)
    }

    /// Prepare only an exact private current token. The owned plain spans
    /// below are proof data, never a native response or an executed edit.
    pub(super) fn private_setup_prepare(ctx: &IdeContext<'_>) -> Option<PrepareRenameResponse> {
        if ctx.uri.scheme() != "file" {
            return None;
        }
        let resident = Self::private_rename_resident(ctx)?;
        let (name, declaration) =
            crate::ide::ReferencesService::private_setup_rename_declaration(ctx)?;
        let (range, spans) =
            crate::ide::references::structural::private_setup_prepare(ctx, &name, declaration)?;
        let mut changes = std::collections::HashMap::new();
        changes.insert(ctx.uri.clone(), spans);
        let proof = WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        };
        if !corsa::RenameScope::native(ctx).admits_authored(&proof)
            || Self::private_rename_resident(ctx)? != resident
        {
            return None;
        }
        Some(PrepareRenameResponse::Range(range))
    }

    fn private_rename_resident(ctx: &IdeContext<'_>) -> Option<(i32, u64)> {
        let document = ctx.state.documents.get(ctx.uri)?;
        (document.uri == *ctx.uri
            && document.language_id == "vue"
            && Arc::ptr_eq(&document.text_snapshot(), &ctx.content))
        .then_some((document.version, document.revision()))
    }
}
