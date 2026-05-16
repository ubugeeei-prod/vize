//! Encoding utilities for semantic tokens.
//!
//! Provides delta encoding, position conversion, and identifier helpers.

use tower_lsp::lsp_types::SemanticToken;

use super::types::AbsoluteToken;

/// Convert byte offset to (line, column) - 1-indexed.
pub(crate) fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 1u32;
    let mut col = 0u32;
    let mut current = 0;

    for ch in source.chars() {
        if current >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current += ch.len_utf8();
    }

    (line, col)
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
