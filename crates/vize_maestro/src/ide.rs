//! IDE features for the LSP server.
//!
//! This module provides core IDE functionality including:
//! - Diagnostics aggregation from multiple sources
//! - Hover information provider
//! - Code completion provider
//! - Go to definition
//! - Find references
//! - Code actions (quick fixes)
//! - Type checking and type information
//! - Rename refactoring
//! - Semantic tokens
//! - Code lens
//! - Workspace symbols
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

pub mod auto_import;
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
pub(crate) mod musea;
pub mod references;
pub mod rename;
pub mod semantic_tokens;
pub mod type_service;
pub mod workspace_symbols;

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
#[cfg(feature = "native")]
pub use jsx::JsxService;
pub use references::ReferencesService;
pub use rename::RenameService;
pub use semantic_tokens::{SemanticTokensService, TokenModifier, TokenType};
pub use type_service::{LspTypeCheckOptions, TypeService};
pub use workspace_symbols::WorkspaceSymbolsService;

use tower_lsp::lsp_types::Url;

use crate::server::ServerState;
use crate::utils::is_standalone_html_path;
use crate::virtual_code::{
    ArtCursorPosition, BlockType, VirtualDocuments, find_art_block_at_offset, find_block_at_offset,
};

// =============================================================================
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
    if name.is_empty() {
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

/// Check if a cursor offset is inside a Vue template expression.
///
/// This covers mustache interpolations and Vue directive attribute values, but
/// deliberately excludes plain text nodes and static attribute values.
pub(crate) fn is_in_vue_template_expression(content: &str, offset: usize) -> bool {
    if content.is_empty() {
        return false;
    }

    let mut offset = offset.min(content.len());
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }

    if is_in_mustache_expression(content, offset) {
        return true;
    }

    is_in_vue_directive_attribute_value(content, offset)
}

fn is_in_mustache_expression(content: &str, offset: usize) -> bool {
    let before = &content[..offset];
    let Some(mustache_start) = before.rfind("{{") else {
        return false;
    };

    let closed_before_cursor = before
        .rfind("}}")
        .is_some_and(|mustache_end| mustache_end > mustache_start);
    if closed_before_cursor {
        return false;
    }

    content[offset..].contains("}}")
}

fn is_in_vue_directive_attribute_value(content: &str, offset: usize) -> bool {
    let bytes = content.as_bytes();
    let mut pos = offset;
    let mut quote_start = None;

    while pos > 0 {
        let byte = bytes[pos - 1];
        match byte {
            b'"' | b'\'' => {
                quote_start = Some((pos - 1, byte));
                break;
            }
            b'<' | b'>' | b'\n' | b'\r' => return false,
            _ => pos -= 1,
        }
    }

    let Some((quote_start, quote)) = quote_start else {
        return false;
    };
    let Some(relative_quote_end) = content[quote_start + 1..].find(quote as char) else {
        return false;
    };
    let quote_end = quote_start + 1 + relative_quote_end;
    if offset > quote_end {
        return false;
    }

    let mut pos = quote_start;
    while pos > 0 && bytes[pos - 1].is_ascii_whitespace() {
        pos -= 1;
    }
    if pos == 0 || bytes[pos - 1] != b'=' {
        return false;
    }
    pos -= 1;

    while pos > 0 && bytes[pos - 1].is_ascii_whitespace() {
        pos -= 1;
    }
    let attr_end = pos;
    while pos > 0 {
        let byte = bytes[pos - 1];
        if byte.is_ascii_whitespace() || matches!(byte, b'<' | b'>' | b'/') {
            break;
        }
        pos -= 1;
    }

    let attr_name = &content[pos..attr_end];
    attr_name.starts_with(':')
        || attr_name.starts_with('@')
        || attr_name.starts_with('#')
        || attr_name.starts_with("v-")
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
        if after_name == bytes.len()
            || matches!(
                bytes[after_name],
                b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r'
            )
        {
            last = Some(start);
        }
        search_start = after_name;
    }

    last
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
    /// Virtual documents for this file
    pub virtual_docs: Option<dashmap::mapref::one::Ref<'a, Url, VirtualDocuments>>,
}

impl<'a> IdeContext<'a> {
    /// Create a new IDE context.
    ///
    /// This re-fetches the document from the store and materializes its content.
    /// Callers that have already materialized the document content should prefer
    /// [`IdeContext::with_content`] to avoid a redundant document lookup and a
    /// second full Rope→String allocation.
    pub fn new(state: &'a ServerState, uri: &'a Url, offset: usize) -> Option<Self> {
        let content = state.documents.get(uri)?.text();
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
mod tests {
    use super::{
        is_component_tag, kebab_to_pascal, offset_to_position, pascal_to_kebab, position_to_offset,
        token_at_offset, token_span_at_offset,
    };

    #[test]
    fn test_offset_to_position() {
        let content = "line1\nline2\nline3";

        assert_eq!(offset_to_position(content, 0), (0, 0));
        assert_eq!(offset_to_position(content, 5), (0, 5));
        assert_eq!(offset_to_position(content, 6), (1, 0));
        assert_eq!(offset_to_position(content, 8), (1, 2));
        assert_eq!(offset_to_position(content, 12), (2, 0));
    }

    #[test]
    fn test_offset_to_position_counts_utf16_code_units() {
        let content = "const icon = \"😀\";\nconst message = icon";

        assert_eq!(
            offset_to_position(content, "const icon = \"😀".len()),
            (0, 16)
        );
        assert_eq!(
            offset_to_position(content, content.find("message").unwrap()),
            (1, 6)
        );
    }

    #[test]
    fn test_position_to_offset() {
        let content = "line1\nline2\nline3";

        assert_eq!(position_to_offset(content, 0, 0), Some(0));
        assert_eq!(position_to_offset(content, 0, 5), Some(5));
        assert_eq!(position_to_offset(content, 1, 0), Some(6));
        assert_eq!(position_to_offset(content, 1, 2), Some(8));
        assert_eq!(position_to_offset(content, 2, 0), Some(12));
    }

    #[test]
    fn test_position_to_offset_counts_utf16_code_units() {
        let content = "a😀b\nc";

        assert_eq!(position_to_offset(content, 0, 3), Some("a😀".len()));
        assert_eq!(position_to_offset(content, 0, 4), Some("a😀b".len()));
        assert_eq!(position_to_offset(content, 1, 1), Some(content.len()));
    }

    #[test]
    fn test_position_to_offset_rejects_utf16_surrogate_pair_interior() {
        let content = "a😀b";

        assert_eq!(position_to_offset(content, 0, 2), None);
    }

    #[test]
    fn test_kebab_to_pascal() {
        assert_eq!(kebab_to_pascal("my-component"), "MyComponent");
        assert_eq!(kebab_to_pascal("button"), "Button");
        assert_eq!(kebab_to_pascal("v-for-item"), "VForItem");
        assert_eq!(kebab_to_pascal("a-b-c"), "ABC");
    }

    #[test]
    fn test_pascal_to_kebab() {
        assert_eq!(pascal_to_kebab("MyComponent"), "my-component");
        assert_eq!(pascal_to_kebab("Button"), "button");
        assert_eq!(pascal_to_kebab("VForItem"), "v-for-item");
        assert_eq!(pascal_to_kebab("ABC"), "a-b-c");
    }

    #[test]
    fn test_is_component_tag() {
        // PascalCase components
        assert!(is_component_tag("MyComponent"));
        assert!(is_component_tag("Button"));

        // kebab-case components
        assert!(is_component_tag("my-component"));
        assert!(is_component_tag("v-button"));

        // HTML elements (not components)
        assert!(!is_component_tag("div"));
        assert!(!is_component_tag("span"));
        assert!(!is_component_tag("button"));
    }

    #[test]
    fn test_token_span_at_offset_allows_identifier_boundaries() {
        let content = "const message = ref(0)";

        assert_eq!(
            token_span_at_offset(content, 5, |c| c.is_ascii_alphanumeric() || c == b'_'),
            Some((0, 5))
        );
        assert_eq!(
            token_span_at_offset(content, 13, |c| c.is_ascii_alphanumeric() || c == b'_'),
            Some((6, 13))
        );
        assert_eq!(
            token_span_at_offset(content, 15, |c| c.is_ascii_alphanumeric() || c == b'_'),
            None
        );
    }

    #[test]
    fn test_token_at_offset_supports_end_of_file_boundaries() {
        let content = "message";

        assert_eq!(
            token_at_offset(content, content.len(), |c| c.is_ascii_alphanumeric()
                || c == b'_'),
            Some("message".to_string())
        );
    }
}
