//! Rename refactoring provider.
//!
//! Provides rename functionality for:
//! - Template bindings (variables, functions, etc.)
//! - Script identifiers
//! - CSS variables in v-bind()
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::collections::HashMap;

#[cfg(feature = "native")]
use std::sync::Arc;

use tower_lsp::lsp_types::{Position, PrepareRenameResponse, Range, TextEdit, WorkspaceEdit};

#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

use super::IdeContext;
#[cfg(feature = "native")]
use crate::ide::corsa_support;
use crate::virtual_code::{ArtCursorPosition, BlockType};

/// Rename service for identifier renaming across SFC.
pub struct RenameService;

impl RenameService {
    /// Check if rename is valid at the given position.
    pub fn prepare_rename(ctx: &IdeContext) -> Option<PrepareRenameResponse> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        // Check if it's a renameable identifier
        if !Self::is_renameable(&word, ctx) {
            return None;
        }

        // Get the range of the word
        let (start, end) = Self::get_word_range(&ctx.content, ctx.offset)?;
        let range = Self::offset_range_to_lsp(&ctx.content, start, end);

        Some(PrepareRenameResponse::Range(range))
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
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() || !Self::is_valid_identifier(new_name) {
            return None;
        }

        let locations = crate::ide::references::ReferencesService::references(ctx, true)?;
        let text_edits: Vec<TextEdit> = locations
            .into_iter()
            .filter(|location| location.uri == *ctx.uri)
            .map(|location| TextEdit {
                range: location.range,
                new_text: new_name.to_string(),
            })
            .collect();
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
        if !Self::is_valid_identifier(new_name) {
            return None;
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

        corsa_result.or_else(|| Self::rename(ctx, new_name))
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
        let request_path = corsa_support::template_request_path(ctx.uri);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;
        let response = bridge.prepare_rename(&uri, line, character).await.ok()??;
        let response = serde_json::from_value(response).ok()?;
        corsa_support::map_corsa_prepare_rename(ctx, &uri, response)
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
        let relative_offset = info.relative_offset as u32;
        let vts_offset = template
            .source_map
            .to_generated(relative_offset)
            .map(|offset| offset as usize)
            .unwrap_or(relative_offset as usize);
        let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);
        let request_path = corsa_support::art_template_request_path(ctx.uri, info.variant_index);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;
        let response = bridge.prepare_rename(&uri, line, character).await.ok()??;
        let response = serde_json::from_value(response).ok()?;
        corsa_support::map_corsa_prepare_rename(ctx, &uri, response)
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
        let request_path = corsa_support::script_request_path(ctx.uri, is_setup);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &script_doc.content)
            .await
            .ok()?;
        let response = bridge.prepare_rename(&uri, line, character).await.ok()??;
        let response = serde_json::from_value(response).ok()?;
        corsa_support::map_corsa_prepare_rename(ctx, &uri, response)
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
        let request_path = corsa_support::template_request_path(ctx.uri);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;
        let edit = bridge
            .rename(&uri, line, character, new_name)
            .await
            .ok()??;
        let edit = serde_json::from_value(edit).ok()?;
        corsa_support::map_corsa_workspace_edit(ctx, edit)
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
        let relative_offset = info.relative_offset as u32;
        let vts_offset = template
            .source_map
            .to_generated(relative_offset)
            .map(|offset| offset as usize)
            .unwrap_or(relative_offset as usize);
        let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);
        let request_path = corsa_support::art_template_request_path(ctx.uri, info.variant_index);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;
        let edit = bridge
            .rename(&uri, line, character, new_name)
            .await
            .ok()??;
        let edit = serde_json::from_value(edit).ok()?;
        corsa_support::map_corsa_workspace_edit(ctx, edit)
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
        let request_path = corsa_support::script_request_path(ctx.uri, is_setup);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &script_doc.content)
            .await
            .ok()?;
        let edit = bridge
            .rename(&uri, line, character, new_name)
            .await
            .ok()??;
        let edit = serde_json::from_value(edit).ok()?;
        corsa_support::map_corsa_workspace_edit(ctx, edit)
    }

    /// Check if the identifier is renameable.
    fn is_renameable(word: &str, ctx: &IdeContext) -> bool {
        // Don't rename Vue directives
        if word.starts_with("v-") {
            return false;
        }

        // Don't rename keywords
        if Self::is_keyword(word) {
            return false;
        }

        // Don't rename $ globals
        if word.starts_with('$') && Self::is_vue_global(word) {
            return false;
        }

        // Check if it's defined in the script
        if let Some(ref virtual_docs) = ctx.virtual_docs {
            if let Some(ref script_setup) = virtual_docs.script_setup {
                let bindings =
                    crate::virtual_code::extract_simple_bindings(&script_setup.content, true);
                if bindings.iter().any(|b| b == word) {
                    return true;
                }
            }
            if let Some(ref script) = virtual_docs.script {
                let bindings = crate::virtual_code::extract_simple_bindings(&script.content, false);
                if bindings.iter().any(|b| b == word) {
                    return true;
                }
            }
        }

        // Allow renaming any valid identifier in template context
        Self::is_valid_identifier(word)
    }

    /// Get the word at the given offset.
    fn get_word_at_offset(content: &str, offset: usize) -> Option<String> {
        crate::ide::token_at_offset(content, offset, |c| Self::is_ident_char(c as char))
    }

    /// Get the range of the word at offset.
    fn get_word_range(content: &str, offset: usize) -> Option<(usize, usize)> {
        let (start, end) =
            crate::ide::token_span_at_offset(content, offset, |c| Self::is_ident_char(c as char))?;

        // Verify it's a valid identifier start
        if !Self::is_ident_start(content.as_bytes()[start] as char) {
            return None;
        }

        Some((start, end))
    }

    /// Convert byte offset range to LSP Range.
    fn offset_range_to_lsp(content: &str, start: usize, end: usize) -> Range {
        let start_pos = Self::offset_to_position(content, start);
        let end_pos = Self::offset_to_position(content, end);
        Range {
            start: start_pos,
            end: end_pos,
        }
    }

    /// Convert byte offset to LSP Position.
    fn offset_to_position(content: &str, offset: usize) -> Position {
        crate::utils::offset_to_position_str(content, offset)
    }

    /// Check if character can start an identifier.
    fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || c == '$'
    }

    /// Check if character can be part of an identifier.
    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    }

    /// Check if string is a valid identifier.
    fn is_valid_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();
        let Some(first) = chars.next() else {
            return false;
        };

        if !Self::is_ident_start(first) {
            return false;
        }

        chars.all(Self::is_ident_char)
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

    /// Check if word is a Vue global.
    fn is_vue_global(word: &str) -> bool {
        matches!(
            word,
            "$el"
                | "$data"
                | "$props"
                | "$attrs"
                | "$refs"
                | "$slots"
                | "$root"
                | "$parent"
                | "$emit"
                | "$forceUpdate"
                | "$nextTick"
                | "$watch"
                | "$options"
                | "$event"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::RenameService;

    /// Renaming a local binding must not rewrite a same-named prop
    /// *attribute name* on a component usage — that token belongs to the
    /// child's own prop symbol, and rewriting it silently breaks the call
    /// site (#3892). Building rename from the references provider makes the
    /// invariant structural: every edit is a reference (or the declaration).
    #[test]
    fn rename_leaves_same_named_prop_attribute_names_alone() {
        let content = r#"<script setup lang="ts">
import Child from './Child.vue'
const count = 1
</script>

<template>
  <Child :count="count" />
</template>
"#;
        let uri = tower_lsp::lsp_types::Url::parse("file:///ws/Parent.vue").unwrap();
        let state = crate::server::ServerState::new();
        state
            .documents
            .open(uri.clone(), content.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, content);
        let offset = content.find("const count").unwrap() + "const ".len();
        let ctx = crate::ide::IdeContext::new(&state, &uri, offset).unwrap();

        let edit = RenameService::rename(&ctx, "tally").unwrap();
        let edits = &edit.changes.unwrap()[&uri];

        let attr_name_line = content[..content.find(":count=").unwrap()]
            .matches('\n')
            .count() as u32;
        for edit in edits {
            let touches_attr_name = edit.range.start.line == attr_name_line
                && edit.range.start.character
                    == content
                        .lines()
                        .nth(attr_name_line as usize)
                        .unwrap()
                        .find(":count")
                        .unwrap() as u32
                        + 1;
            assert!(
                !touches_attr_name,
                "the :count attribute name must not be renamed: {edits:?}"
            );
        }

        // The invariant that pins the providers together: every rename edit is
        // one of the reference locations.
        let references = crate::ide::references::ReferencesService::references(&ctx, true).unwrap();
        for edit in edits {
            assert!(
                references
                    .iter()
                    .any(|reference| reference.range == edit.range),
                "rename edit {edit:?} is not a reference: {references:?}"
            );
        }
        assert!(
            edits.len() >= 2,
            "the declaration and the binding value must still rename: {edits:?}"
        );
    }

    #[test]
    fn test_get_word_at_offset() {
        let content = "const count = ref(0)";
        assert_eq!(
            RenameService::get_word_at_offset(content, 6),
            Some("count".to_string())
        );
        assert_eq!(
            RenameService::get_word_at_offset(content, 14),
            Some("ref".to_string())
        );
        assert_eq!(
            RenameService::get_word_at_offset(content, 11),
            Some("count".to_string())
        );
        assert_eq!(RenameService::get_word_at_offset(content, 12), None);
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(RenameService::is_valid_identifier("count"));
        assert!(RenameService::is_valid_identifier("_private"));
        assert!(RenameService::is_valid_identifier("$refs"));
        assert!(!RenameService::is_valid_identifier("123abc"));
        assert!(!RenameService::is_valid_identifier(""));
    }

    #[test]
    fn test_is_keyword() {
        assert!(RenameService::is_keyword("const"));
        assert!(RenameService::is_keyword("function"));
        assert!(!RenameService::is_keyword("count"));
    }

    #[test]
    fn test_offset_range_to_lsp_counts_utf16_code_units() {
        let content = "const emoji = \"😀\"; const message = ref(emoji)";
        let start = content.find("message").unwrap();
        let end = start + "message".len();
        let range = RenameService::offset_range_to_lsp(content, start, end);

        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 26);
        assert_eq!(range.end.character, 33);
    }
}
