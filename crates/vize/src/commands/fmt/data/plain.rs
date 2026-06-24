//! Conservative formatting for plain config/doc files: YAML and Markdown.
//!
//! These formats have no comment-preserving parser available here, so the
//! transforms are deliberately lossless:
//!
//! - **YAML** (`.yaml`/`.yml`): normalize line endings to `\n` and ensure a
//!   single trailing newline. Line content is preserved byte-for-byte, so
//!   comments, anchors/aliases, and block scalars (`|`/`>`, including
//!   `+`-chomped trailing blanks) are never altered.
//! - **Markdown** (`.md`/`.markdown`): the above, plus trailing-whitespace
//!   trimming outside fenced (```` ``` ````/`~~~`) and indented code blocks,
//!   preserving two-space hard line breaks.
//!
//! Richer, structure-aware formatting is a follow-up that needs real parsers.

use vize_carton::String;
use vize_glyph::FormatResult;

/// Format a YAML document (lossless newline normalization).
pub(super) fn format_yaml(source: &str) -> FormatResult {
    finish(normalize_yaml(source), source)
}

/// Format a Markdown document (newline normalization + safe trailing-ws trim).
pub(super) fn format_markdown(source: &str) -> FormatResult {
    finish(normalize_markdown(source), source)
}

fn finish(code: String, source: &str) -> FormatResult {
    FormatResult {
        changed: code.as_str() != source,
        code,
    }
}

/// Convert `\r\n` and lone `\r` line endings to `\n`.
fn to_lf(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(ch);
        }
    }
    out
}

fn ensure_final_newline(mut text: String) -> String {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

fn normalize_yaml(source: &str) -> String {
    ensure_final_newline(to_lf(source))
}

fn normalize_markdown(source: &str) -> String {
    let lf = to_lf(source);
    let mut out = String::with_capacity(lf.len());
    let mut fence: Option<char> = None;
    for (index, line) in lf.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let stripped = line.trim_start();
        let fence_char = fence_marker(stripped);
        if let Some(marker) = fence_char {
            match fence {
                Some(open) if open == marker => fence = None,
                None => fence = Some(marker),
                Some(_) => {}
            }
            out.push_str(line);
        } else if fence.is_some() || is_indented_code(line) {
            out.push_str(line);
        } else {
            out.push_str(trim_trailing(line).as_str());
        }
    }
    ensure_final_newline(out)
}

/// The fence character if `stripped` opens/closes a fenced code block.
fn fence_marker(stripped: &str) -> Option<char> {
    if stripped.starts_with("```") {
        Some('`')
    } else if stripped.starts_with("~~~") {
        Some('~')
    } else {
        None
    }
}

/// CommonMark indented code blocks start at >= 4 spaces or a tab. Treating any
/// such line as code is conservative: it never strips significant whitespace.
fn is_indented_code(line: &str) -> bool {
    line.starts_with("    ") || line.starts_with('\t')
}

/// Trim trailing whitespace, preserving a hard line break (exactly two trailing
/// spaces). Three-or-more or mixed trailing whitespace is trimmed in full.
fn trim_trailing(line: &str) -> String {
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        return String::default();
    }
    let mut out = String::from(trimmed);
    if &line[trimmed.len()..] == "  " {
        out.push_str("  ");
    }
    out
}

#[cfg(test)]
mod tests;
