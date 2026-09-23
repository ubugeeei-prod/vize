//! Rename refactoring provider.
//!
//! Provides rename functionality for:
//! - Template bindings (variables, functions, etc.)
//! - Script identifiers
//! - CSS variables in v-bind()
#![expect(
    clippy::disallowed_types,
    reason = "tower-lsp lsp_types take std String values"
)]
#![cfg_attr(
    feature = "native",
    expect(
        clippy::disallowed_methods,
        reason = "to_string builds lsp_types Strings"
    )
)]

#[cfg(feature = "native")]
mod canonical;
#[cfg(all(test, feature = "native"))]
mod corsa_event_variants_tests;
#[cfg(all(test, feature = "native"))]
mod corsa_model_tests;
#[cfg(all(test, feature = "native", unix))]
pub(in crate::ide) mod corsa_session_tests;
// Plain `cfg(test)` lets clippy's test exemptions apply inside the module.
#[cfg(test)]
#[cfg(feature = "native")]
mod corsa_tests;

use std::collections::HashMap;
#[cfg(feature = "native")]
use std::sync::Arc;
use tower_lsp::lsp_types::{PrepareRenameResponse, WorkspaceEdit};

#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

use super::IdeContext;
#[cfg(feature = "native")]
use crate::ide::corsa_support as corsa;
#[cfg(feature = "native")]
use crate::virtual_code::{ArtCursorPosition, BlockType};

/// Rename service for identifier renaming across SFC.
pub struct RenameService;

impl RenameService {
    /// Check if rename is valid at the given position.
    pub fn prepare_rename(ctx: &IdeContext) -> Option<PrepareRenameResponse> {
        crate::ide::references::structural::prepare_rename(ctx).map(PrepareRenameResponse::Range)
    }

    /// Perform rename operation.
    ///
    /// Edits are built from the references provider rather than a textual
    /// sweep, so the two can never disagree about symbol identity: the sweep
    /// used to rewrite `:count` *attribute names* — the child component's own
    /// prop — when renaming an unrelated local `count`, silently breaking the
    /// call sites, while references classified the same spans correctly by
    /// searching template *expressions* only (#3892).
    pub fn rename(ctx: &IdeContext, new_name: &str) -> Option<WorkspaceEdit> {
        if !Self::is_valid_identifier(new_name) || Self::is_keyword(new_name) {
            return None;
        }
        let text_edits = crate::ide::references::structural::rename(ctx, new_name)?;
        if text_edits.is_empty() {
            return None;
        }

        let mut changes = HashMap::new();
        changes.insert(ctx.uri.clone(), text_edits);

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }

    /// Check rename availability using Corsa when possible, with synchronous fallback.
    #[cfg(feature = "native")]
    pub async fn prepare_rename_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<PrepareRenameResponse> {
        match canonical::prepare(ctx, corsa_bridge.as_deref()).await {
            canonical::Answer::Available(response) => return response,
            canonical::Answer::Unavailable => {}
        }
        let corsa_result = match ctx.block_type? {
            BlockType::Template => {
                Self::prepare_template_rename_with_corsa(ctx, corsa_bridge.as_deref()).await
            }
            BlockType::Script | BlockType::ScriptSetup => {
                Self::prepare_script_rename_with_corsa(
                    ctx,
                    matches!(ctx.block_type, Some(BlockType::ScriptSetup)),
                    corsa_bridge.as_deref(),
                )
                .await
            }
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::prepare_art_variant_rename_with_corsa(ctx, info, corsa_bridge.as_deref())
                    .await
            }
            BlockType::Style(_) | BlockType::Art(_) => None,
        };

        corsa_result.or_else(|| Self::prepare_rename(ctx))
    }

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

    #[cfg(feature = "native")]
    async fn prepare_template_rename_with_corsa(
        ctx: &IdeContext<'_>,
        bridge: Option<&CorsaBridge>,
    ) -> Option<PrepareRenameResponse> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let template = virtual_docs.template.as_ref()?;
        let vts_offset =
            crate::ide::hover::HoverService::sfc_to_virtual_ts_offset(ctx, ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);
        let request_path = corsa::template_request_path(ctx.uri);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;
        let response = bridge.prepare_rename(&uri, line, character).await.ok()??;
        let response = serde_json::from_value(response).ok()?;
        corsa::map_corsa_prepare_rename(ctx, &uri, response)
    }

    #[cfg(feature = "native")]
    async fn prepare_art_variant_rename_with_corsa(
        ctx: &IdeContext<'_>,
        info: &crate::virtual_code::ArtVariantInfo,
        bridge: Option<&CorsaBridge>,
    ) -> Option<PrepareRenameResponse> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let template = virtual_docs.art_template(info.variant_index)?;
        let vts_offset = template.source_map.to_generated(ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);
        let request_path = corsa::art_template_request_path(ctx.uri, info.variant_index);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;
        let response = bridge.prepare_rename(&uri, line, character).await.ok()??;
        let response = serde_json::from_value(response).ok()?;
        corsa::map_corsa_prepare_rename(ctx, &uri, response)
    }

    #[cfg(feature = "native")]
    async fn prepare_script_rename_with_corsa(
        ctx: &IdeContext<'_>,
        is_setup: bool,
        bridge: Option<&CorsaBridge>,
    ) -> Option<PrepareRenameResponse> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let script_doc = if is_setup {
            virtual_docs.script_setup.as_ref()
        } else {
            virtual_docs.script.as_ref()
        }?;
        let vts_offset =
            crate::ide::hover::HoverService::sfc_to_virtual_ts_script_offset(ctx, ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&script_doc.content, vts_offset);
        let request_path = corsa::script_request_path(ctx.uri, is_setup);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &script_doc.content)
            .await
            .ok()?;
        let response = bridge.prepare_rename(&uri, line, character).await.ok()??;
        let response = serde_json::from_value(response).ok()?;
        corsa::map_corsa_prepare_rename(ctx, &uri, response)
    }

    #[cfg(feature = "native")]
    async fn rename_template_with_corsa(
        ctx: &IdeContext<'_>,
        new_name: &str,
        bridge: Option<&CorsaBridge>,
    ) -> Option<WorkspaceEdit> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let template = virtual_docs.template.as_ref()?;
        let vts_offset =
            crate::ide::hover::HoverService::sfc_to_virtual_ts_offset(ctx, ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);
        let request_path = corsa::template_request_path(ctx.uri);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;
        let edit = bridge
            .rename(&uri, line, character, new_name)
            .await
            .ok()??;
        let edit = serde_json::from_value(edit).ok()?;
        corsa::map_corsa_workspace_edit(ctx, edit)
    }

    #[cfg(feature = "native")]
    async fn rename_art_variant_with_corsa(
        ctx: &IdeContext<'_>,
        info: &crate::virtual_code::ArtVariantInfo,
        new_name: &str,
        bridge: Option<&CorsaBridge>,
    ) -> Option<WorkspaceEdit> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let template = virtual_docs.art_template(info.variant_index)?;
        let vts_offset = template.source_map.to_generated(ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);
        let request_path = corsa::art_template_request_path(ctx.uri, info.variant_index);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;
        let edit = bridge
            .rename(&uri, line, character, new_name)
            .await
            .ok()??;
        let edit = serde_json::from_value(edit).ok()?;
        corsa::map_corsa_workspace_edit(ctx, edit)
    }

    #[cfg(feature = "native")]
    async fn rename_script_with_corsa(
        ctx: &IdeContext<'_>,
        new_name: &str,
        is_setup: bool,
        bridge: Option<&CorsaBridge>,
    ) -> Option<WorkspaceEdit> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let script_doc = if is_setup {
            virtual_docs.script_setup.as_ref()
        } else {
            virtual_docs.script.as_ref()
        }?;
        let vts_offset =
            crate::ide::hover::HoverService::sfc_to_virtual_ts_script_offset(ctx, ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&script_doc.content, vts_offset);
        let request_path = corsa::script_request_path(ctx.uri, is_setup);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &script_doc.content)
            .await
            .ok()?;
        let edit = bridge
            .rename(&uri, line, character, new_name)
            .await
            .ok()??;
        let edit = serde_json::from_value(edit).ok()?;
        corsa::map_corsa_workspace_edit(ctx, edit)
    }

    /// Check JavaScript identifier syntax, including Unicode identifiers.
    fn is_valid_identifier(s: &str) -> bool {
        oxc_syntax::identifier::is_identifier_name(s)
    }

    /// Check if word is a JavaScript keyword.
    fn is_keyword(word: &str) -> bool {
        matches!(
            word,
            "break"
                | "case"
                | "catch"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "in"
                | "instanceof"
                | "new"
                | "return"
                | "switch"
                | "this"
                | "throw"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
                | "class"
                | "const"
                | "enum"
                | "export"
                | "extends"
                | "import"
                | "super"
                | "implements"
                | "interface"
                | "let"
                | "package"
                | "private"
                | "protected"
                | "public"
                | "static"
                | "yield"
                | "true"
                | "false"
                | "null"
                | "undefined"
                | "async"
                | "await"
                | "of"
        )
    }
}

#[cfg(test)]
mod tests;
