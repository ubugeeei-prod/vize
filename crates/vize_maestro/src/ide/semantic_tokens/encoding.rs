//! Encoding utilities for semantic tokens.
//!
//! Provides delta encoding, position conversion, and identifier helpers.

use tower_lsp::lsp_types::SemanticToken;

use super::types::AbsoluteToken;

// Shared, UTF-16-correct line-index utility lives in `vize_carton::line_index`
// (#1389). Both the binary-search [`LineIndex`] and the single-shot
// [`offset_to_line_col`] helper are re-exported here so existing call sites and
// the perf characteristics are unchanged.
pub(crate) use vize_carton::line_index::{LineIndex, offset_to_line_col};

/// Return the LSP token length for text, measured in UTF-16 code units.
pub(crate) fn utf16_len(text: &str) -> u32 {
    text.encode_utf16().count() as u32
}

/// Encode tokens using delta encoding.
pub(crate) fn encode_tokens(tokens: &[AbsoluteToken]) -> Vec<SemanticToken> {
    let mut result = Vec::with_capacity(tokens.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for token in tokens {
        let delta_line = token.line - prev_line;
        let delta_start = if delta_line == 0 {
            token.start - prev_start
        } else {
            token.start
        };

        result.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: token.modifiers,
        });

        prev_line = token.line;
        prev_start = token.start;
    }

    result
}

/// Check if character can start an identifier.
pub(crate) fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$'
}

/// Check if character can be part of an identifier.
pub(crate) fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

/// Check if identifier is a keyword or literal (used in tests).
#[cfg(test)]
pub(crate) fn is_keyword_or_literal(s: &str) -> bool {
    matches!(
        s,
        "true"
            | "false"
            | "null"
            | "undefined"
            | "this"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "const"
            | "let"
            | "var"
            | "function"
            | "class"
            | "return"
            | "break"
            | "continue"
            | "new"
            | "typeof"
            | "in"
            | "of"
            | "instanceof"
            | "async"
            | "await"
    )
}
