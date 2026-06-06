//! Encoding utilities for semantic tokens.
//!
//! Provides delta encoding, position conversion, and identifier helpers.

use tower_lsp::lsp_types::SemanticToken;

use super::types::AbsoluteToken;

/// Precomputed byte offsets of every line start in a source string.
///
/// Building this once and reusing it turns the per-call O(offset) scan of
/// [`offset_to_line_col`] into a binary search over line starts plus a short
/// UTF-16 column scan within the target line. Token collectors that map many
/// offsets against the same `content` build it once and thread it through.
///
/// `line_col` reproduces [`offset_to_line_col`] byte-for-byte, including the
/// UTF-16 code-unit column semantics LSP requires.
pub(crate) struct LineIndex<'a> {
    source: &'a str,
    /// Byte offset of the start of each line. `line_starts[0]` is always `0`.
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    /// Build a line index in a single pass over `source`.
    pub(crate) fn new(source: &'a str) -> Self {
        let mut line_starts = Vec::with_capacity(source.len() / 32 + 1);
        line_starts.push(0);
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self {
            source,
            line_starts,
        }
    }

    /// Convert a byte offset to (line, column), both 0-indexed for LSP.
    ///
    /// `column` is measured in UTF-16 code units. Offsets past the end of the
    /// source are clamped, matching the natural EOF behavior of the legacy scan.
    pub(crate) fn line_col(&self, offset: usize) -> (u32, u32) {
        let offset = offset.min(self.source.len());

        // Greatest line start <= offset.
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(next) => next - 1,
        };
        let line_start = self.line_starts[line];

        // Sum UTF-16 code units of every character on this line whose byte
        // start is before `offset`. Bounded to the target line, and tolerant of
        // an `offset` that falls mid-character (it counts the partial char,
        // matching the legacy scan's break-at-top behavior) so it never panics.
        let mut col = 0u32;
        for (i, ch) in self.source[line_start..].char_indices() {
            if line_start + i >= offset {
                break;
            }
            col += ch.len_utf16() as u32;
        }

        (line as u32, col)
    }
}

/// Convert byte offset to (line, column), both 0-indexed for LSP.
///
/// Thin wrapper over [`LineIndex`] for single-shot callers. Hot paths that map
/// many offsets against the same content should build a [`LineIndex`] once and
/// call [`LineIndex::line_col`] instead.
pub(crate) fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    LineIndex::new(source).line_col(offset)
}

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
