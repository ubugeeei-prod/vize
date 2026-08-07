//! IDE features for the LSP server. The module list below is the
//! authoritative inventory:
//! - Correctness: diagnostics, type checking and type information
//! - Authoring: hover, completion, definition, references, code actions, rename, linked editing
//! - Structure: document/workspace symbols, selection ranges, semantic tokens, inlay hints
//! - Ecosystem: router/i18n awareness, file rename, auto-import, code lens, document links
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]
pub mod auto_import;
pub mod auto_insert;
pub mod code_action;
pub mod code_lens;
pub mod completion;
mod corsa_support;
pub mod cursor_context;
pub mod definition;
pub mod diagnostics;
pub mod document_highlight;
pub mod document_link;
pub(crate) mod ecosystem;
pub mod file_rename;
pub mod hover;
pub mod inlay_hint;
pub mod jsx;
pub mod linked_editing;
pub(crate) mod markup;
pub(crate) mod musea;
pub mod references;
pub mod rename;
pub mod selection_range;
pub mod semantic_tokens;
pub(crate) mod sfc_region;
pub(crate) mod tag_pair;
mod template_expression;
pub(crate) mod tsconfig_paths;
pub mod type_service;
pub mod workspace_symbols;
pub use auto_insert::AutoInsertService;
pub use code_action::CodeActionService;
pub use code_lens::CodeLensService;
pub use completion::{CompletionService, TRIGGER_CHARACTERS, trigger_characters};
pub use cursor_context::CursorContext;
pub use definition::{BindingKind, BindingLocation, DefinitionService};
pub use diagnostics::{DiagnosticBuilder, DiagnosticService, Severity, sources};
pub use document_highlight::DocumentHighlightService;
pub use document_link::DocumentLinkService;
pub use file_rename::FileRenameService;
pub use hover::{HoverBuilder, HoverService};
pub use inlay_hint::InlayHintService;
pub use jsx::{
    JsxCodeActionService, JsxDocumentSymbolsService, JsxScopedStyleService,
    JsxSemanticTokensService,
};
#[cfg(feature = "native")]
pub use jsx::{JsxReferencesService, JsxRenameService, JsxService};
pub use references::ReferencesService;
pub use rename::RenameService;
pub use selection_range::SelectionRangeService;
pub use semantic_tokens::{SemanticTokensService, TokenModifier, TokenType};
pub(crate) use template_expression::is_in_vue_template_expression;
pub use type_service::{LspTypeCheckOptions, TypeService};
pub use workspace_symbols::WorkspaceSymbolsService;

use tower_lsp::lsp_types::Url;

use crate::server::ServerState;
use crate::utils::is_standalone_html_path;
use crate::virtual_code::{
    ArtCursorPosition, BlockType, VirtualDocuments, find_art_block_at_offset, find_block_at_offset,
};

// Position conversion utilities
// =============================================================================

/// Convert byte offset to (line, character) position in a document.
#[inline]
pub fn offset_to_position(content: &str, offset: usize) -> (u32, u32) {
    let position = crate::utils::offset_to_position_str(content, offset);
    (position.line, position.character)
}

/// Convert (line, character) position to byte offset in a document.
#[inline]
pub fn position_to_offset(content: &str, line: u32, character: u32) -> Option<usize> {
    fn offset_in_line(content: &str, line_start: usize, character: u32) -> Option<usize> {
        let mut utf16_units = 0u32;

        for (relative_offset, ch) in content[line_start..].char_indices() {
            if ch == '\n' {
                return (utf16_units == character).then_some(line_start + relative_offset);
            }
            if utf16_units == character {
                return Some(line_start + relative_offset);
            }

            let next_utf16_units = utf16_units + ch.len_utf16() as u32;
            if character < next_utf16_units {
                return None;
            }
            utf16_units = next_utf16_units;
        }

        (utf16_units == character).then_some(content.len())
    }

    let mut current_line = 0u32;
    let mut line_start = 0usize;

    for (offset, ch) in content.char_indices() {
        if current_line == line {
            return offset_in_line(content, line_start, character);
        }

        if ch == '\n' {
            current_line += 1;
            line_start = offset + ch.len_utf8();
        }
    }

    if current_line == line {
        return offset_in_line(content, line_start, character);
    }

    None
}

// =============================================================================
// Component name conversion utilities
// =============================================================================

/// Convert kebab-case to PascalCase.
/// Example: "my-component" -> "MyComponent"
pub fn kebab_to_pascal(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut capitalize_next = true;

    for ch in name.chars() {
        if ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert PascalCase to kebab-case.
/// Example: "MyComponent" -> "my-component"
pub fn pascal_to_kebab(name: &str) -> String {
    let mut result = String::with_capacity(name.len() + 4);

    for (i, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}

/// Check if a tag name is a component (starts with uppercase or contains hyphen).
#[inline]
pub fn is_component_tag(name: &str) -> bool {
    if name.is_empty() || vize_carton::is_native_tag(name) {
        return false;
    }
    let Some(first) = name.chars().next() else {
        return false;
    };
    first.is_ascii_uppercase() || name.contains('-')
}

/// Resolve the token span around a cursor offset.
///
/// If the cursor is placed just after a token, the previous character is used
/// so LSP requests at identifier boundaries still resolve the symbol.
pub(crate) fn token_span_at_offset<F>(
    content: &str,
    offset: usize,
    is_token_char: F,
) -> Option<(usize, usize)>
where
    F: Fn(u8) -> bool,
{
    let bytes = content.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut cursor = offset.min(bytes.len());
    if cursor == bytes.len() {
        cursor = cursor.saturating_sub(1);
    }

    if !is_token_char(bytes[cursor]) {
        if cursor > 0 && is_token_char(bytes[cursor - 1]) {
            cursor -= 1;
        } else {
            return None;
        }
    }

    let mut start = cursor;
    while start > 0 && is_token_char(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = cursor + 1;
    while end < bytes.len() && is_token_char(bytes[end]) {
        end += 1;
    }

    Some((start, end))
}

/// Resolve the token string around a cursor offset.
pub(crate) fn token_at_offset<F>(content: &str, offset: usize, is_token_char: F) -> Option<String>
where
    F: Fn(u8) -> bool,
{
    let (start, end) = token_span_at_offset(content, offset, is_token_char)?;
    Some(content[start..end].to_string())
}

fn standalone_html_block_at_offset(content: &str, offset: usize) -> BlockType {
    if is_inside_raw_html_element(content, offset, "script") {
        BlockType::Script
    } else if is_inside_raw_html_element(content, offset, "style") {
        BlockType::Style(0)
    } else {
        BlockType::Template
    }
}

fn is_inside_raw_html_element(content: &str, offset: usize, tag_name: &str) -> bool {
    let cursor = offset.min(content.len());
    let before = content[..cursor].to_ascii_lowercase();
    let Some(open_start) = last_start_tag(&before, tag_name) else {
        return false;
    };

    let close_needle = if tag_name == "script" {
        "</script"
    } else {
        "</style"
    };
    if before
        .rfind(close_needle)
        .is_some_and(|close_start| close_start > open_start)
    {
        return false;
    }

    before[open_start..].contains('>')
}

fn last_start_tag(content: &str, tag_name: &str) -> Option<usize> {
    let needle = if tag_name == "script" {
        "<script"
    } else {
        "<style"
    };
    let bytes = content.as_bytes();
    let mut search_start = 0;
    let mut last = None;

    while let Some(relative) = content[search_start..].find(needle) {
        let start = search_start + relative;
        let after_name = start + needle.len();
        if (after_name == bytes.len()
            || matches!(
                bytes[after_name],
                b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r'
            ))
            && !is_inside_html_comment_at(content, start)
        {
            last = Some(start);
        }
        search_start = after_name;
    }

    last
}

fn is_inside_html_comment_at(content: &str, offset: usize) -> bool {
    let before = &content[..offset.min(content.len())];
    let Some(open) = before.rfind("<!--") else {
        return false;
    };
    before.rfind("-->").is_none_or(|close| open > close)
}

/// Context for IDE operations.
pub struct IdeContext<'a> {
    /// Server state
    pub state: &'a ServerState,
    /// Document URI
    pub uri: &'a Url,
    /// Document content
    pub content: String,
    /// Cursor offset in the document
    pub offset: usize,
    /// Which block the cursor is in
    pub block_type: Option<BlockType>,
    /// Virtual documents for this file. An owned snapshot, never a `DashMap` shard guard: `&IdeContext` crosses the `.await` points of hover, completion, definition, references and rename, where a live guard hangs the server — see [`ServerState::get_virtual_docs`] (#3377).
    pub virtual_docs: Option<std::sync::Arc<VirtualDocuments>>,
}

impl<'a> IdeContext<'a> {
    /// Create a new IDE context.
    ///
    /// This re-fetches the document from the store and materializes its content.
    /// Callers that have already materialized the document content should prefer
    /// [`IdeContext::with_content`] to avoid a redundant document lookup and a
    /// second full Rope→String allocation.
    pub fn new(state: &'a ServerState, uri: &'a Url, offset: usize) -> Option<Self> {
        let content = state.documents.text(uri)?;
        Some(Self::with_content(state, uri, offset, content))
    }

    /// Create a new IDE context from already-materialized document content.
    ///
    /// Reuses the provided `content` instead of re-reading the document from the
    /// store, avoiding a redundant `DashMap` lookup and a second full
    /// Rope→String allocation per request.
    pub fn with_content(
        state: &'a ServerState,
        uri: &'a Url,
        offset: usize,
        content: String,
    ) -> Self {
        // Determine block type
        let block_type = if uri.path().ends_with(".art.vue") {
            // For art files, use art-specific block detection
            find_art_block_at_offset(&content, offset)
        } else if is_standalone_html_path(uri.path()) {
            Some(standalone_html_block_at_offset(&content, offset))
        } else {
            // Parse SFC to determine block type
            let options = vize_atelier_sfc::SfcParseOptions {
                filename: uri.path().to_string().into(),
                ..Default::default()
            };
            if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&content, options) {
                find_block_at_offset(&descriptor, offset)
            } else {
                None
            }
        };

        let virtual_docs = state.get_virtual_docs(uri);

        Self {
            state,
            uri,
            content,
            offset,
            block_type,
            virtual_docs,
        }
    }

    /// Effective Vue dialect for this document.
    ///
    /// Delegates to [`ServerState::document_dialect`]: an explicit `dialect`
    /// config key wins, otherwise the structural petite-vue detection memoized
    /// on the open document is used (no per-request re-scan).
    #[inline]
    pub fn dialect(&self) -> vize_carton::dialect::VueDialect {
        self.state.document_dialect(self.uri, &self.content)
    }

    /// Check if cursor is in template block.
    #[inline]
    pub fn is_in_template(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Template))
    }

    /// Check if cursor is in script block.
    #[inline]
    pub fn is_in_script(&self) -> bool {
        matches!(
            self.block_type,
            Some(BlockType::Script) | Some(BlockType::ScriptSetup)
        )
    }

    /// Check if cursor is in style block.
    #[inline]
    pub fn is_in_style(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Style(_)))
    }

    /// Check if cursor is in an art custom block.
    #[inline]
    pub fn is_in_art(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Art(_)))
    }

    /// Check if cursor is in an art variant template.
    #[inline]
    pub fn is_in_art_variant_template(&self) -> bool {
        matches!(
            self.block_type,
            Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
        )
    }
}

#[cfg(test)]
mod tests;
