//! IDE features for the LSP server. Module inventory:
//! - Correctness/authoring: diagnostics, type info, hover, completion, navigation, rename, linked editing
//! - Structure: document/workspace symbols, selection ranges, semantic tokens, inlay hints
//! - Ecosystem: router/i18n awareness, file rename, auto-import, code lens, document links
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]
pub mod auto_import;
pub mod auto_insert;
pub mod call_hierarchy;
pub mod code_action;
pub mod code_lens;
pub mod completion;
mod context;
mod corsa_support;
#[cfg(feature = "native")]
mod native_code_actions;
#[cfg(all(test, feature = "native"))]
pub(crate) use corsa_support::{canonical_request_path, request_file_uri};
pub mod cursor_context;
pub mod declaration;
pub mod definition;
pub mod diagnostics;
pub mod document_highlight;
pub mod document_link;
pub(crate) mod ecosystem;
pub mod file_rename;
pub mod hover;
pub mod implementation;
pub mod inlay_hint;
pub mod jsx;
pub mod linked_editing;
pub(crate) mod markup;
pub(crate) mod musea;
pub(crate) mod pug;
pub mod references;
pub mod rename;
pub mod selection_range;
pub mod semantic_tokens;
pub(crate) mod sfc_region;
pub mod signature_help;
pub(crate) mod tag_pair;
mod template_expression;
pub(crate) mod template_ref;
pub(crate) mod template_scope;
pub(crate) mod tsconfig_paths;
pub mod type_definition;
pub mod type_service;
pub mod workspace_symbols;
pub use auto_insert::AutoInsertService;
pub use call_hierarchy::CallHierarchyService;
pub use code_action::CodeActionService;
pub use code_lens::CodeLensService;
pub use completion::{CompletionService, TRIGGER_CHARACTERS, trigger_characters};
pub use context::IdeContext;
pub use cursor_context::CursorContext;
pub use declaration::DeclarationService;
pub use definition::{BindingKind, BindingLocation, DefinitionService};
pub use diagnostics::{DiagnosticBuilder, DiagnosticService, Severity, sources};
pub use document_highlight::DocumentHighlightService;
pub use document_link::DocumentLinkService;
pub use file_rename::FileRenameService;
pub use hover::{HoverBuilder, HoverService};
pub use implementation::ImplementationService;
pub use inlay_hint::InlayHintService;
pub use jsx::{
    JsxCodeActionService, JsxDocumentSymbolsService, JsxScopedStyleService,
    JsxSemanticTokensService,
};
#[cfg(feature = "native")]
pub use jsx::{
    JsxDeclarationService, JsxImplementationService, JsxReferencesService, JsxRenameService,
    JsxService, JsxTypeDefinitionService,
};
pub use references::ReferencesService;
pub use rename::RenameService;
pub use selection_range::SelectionRangeService;
pub use semantic_tokens::{SemanticTokensService, TokenModifier, TokenType};
pub use signature_help::SignatureHelpService;
pub(crate) use template_expression::is_in_vue_template_expression;
pub use type_definition::TypeDefinitionService;
pub use type_service::{LspTypeCheckOptions, TypeService};
pub use workspace_symbols::WorkspaceSymbolsService;

use crate::virtual_code::BlockType;

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
    vize_s0::line_index::LineBreaks::Lsp.position_to_offset(content, line, character)
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

/// Candidate local binding names for a component tag.
pub(crate) fn component_name_candidates(name: &str) -> Vec<String> {
    let Some(first) = name.chars().next() else {
        return Vec::new();
    };
    if !name.contains('-') && !first.is_ascii_uppercase() {
        return vec![name.to_string()];
    }

    let pascal = kebab_to_pascal(name);
    let camel = lower_first_ascii(&pascal);
    let mut names = Vec::with_capacity(3);
    push_unique_name(&mut names, name.to_string());
    push_unique_name(&mut names, pascal);
    push_unique_name(&mut names, camel);
    names
}

fn lower_first_ascii(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut result = String::with_capacity(name.len());
    result.push(first.to_ascii_lowercase());
    result.extend(chars);
    result
}

fn push_unique_name(names: &mut Vec<String>, candidate: String) {
    if !names.iter().any(|name| name == &candidate) {
        names.push(candidate);
    }
}

/// Check if a tag name is a component (starts with uppercase or contains hyphen).
#[inline]
pub fn is_component_tag(name: &str) -> bool {
    if name.is_empty() || vize_s0::is_native_tag(name) {
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
    let mut cursor = offset.min(bytes.len().checked_sub(1)?);
    let is_token_at = |index: usize| bytes.get(index).is_some_and(|&byte| is_token_char(byte));

    if !is_token_at(cursor) {
        if cursor > 0 && is_token_at(cursor - 1) {
            cursor -= 1;
        } else {
            return None;
        }
    }

    let (before, after) = bytes.split_at_checked(cursor)?;
    let start = before
        .iter()
        .rposition(|&byte| !is_token_char(byte))
        .map_or(0, |index| index + 1);
    let end = cursor
        + after
            .iter()
            .skip(1)
            .position(|&byte| !is_token_char(byte))
            .map_or(after.len(), |index| index + 1);

    Some((start, end))
}

/// Resolve the token string around a cursor offset.
pub(crate) fn token_at_offset<F>(content: &str, offset: usize, is_token_char: F) -> Option<String>
where
    F: Fn(u8) -> bool,
{
    let (start, end) = token_span_at_offset(content, offset, is_token_char)?;
    content.get(start..end).map(str::to_string)
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
    let Some(before) = content.get(..cursor).map(str::to_ascii_lowercase) else {
        return false;
    };
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

    before
        .get(open_start..)
        .is_some_and(|open_tag| open_tag.contains('>'))
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

    while let Some(relative) = content
        .get(search_start..)
        .and_then(|rest| rest.find(needle))
    {
        let start = search_start + relative;
        let after_name = start + needle.len();
        if bytes
            .get(after_name)
            .is_none_or(|byte| matches!(byte, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r'))
            && !is_inside_html_comment_at(content, start)
        {
            last = Some(start);
        }
        search_start = after_name;
    }

    last
}

fn is_inside_html_comment_at(content: &str, offset: usize) -> bool {
    let Some(before) = content.get(..offset.min(content.len())) else {
        return false;
    };
    let Some(open) = before.rfind("<!--") else {
        return false;
    };
    before.rfind("-->").is_none_or(|close| open > close)
}

#[cfg(test)]
pub(crate) mod tests;
