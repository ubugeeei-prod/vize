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

use vize_glyph::FormatResult;
use vize_l0::String;

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
    let mut fence: Option<FenceMarker> = None;
    for (index, line) in lf.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let stripped = line.trim_start();
        let marker = fence_marker(stripped);
        if let Some(marker) = marker {
            match fence {
                Some(open) if marker.closes(open) => fence = None,
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

#[derive(Clone, Copy)]
struct FenceMarker {
    ch: char,
    len: usize,
    blank_tail: bool,
}

impl FenceMarker {
    fn closes(self, open: Self) -> bool {
        self.ch == open.ch && self.len >= open.len && self.blank_tail
    }
}

/// The fence marker if `stripped` opens/closes a fenced code block.
fn fence_marker(stripped: &str) -> Option<FenceMarker> {
    let ch = match stripped.as_bytes().first().copied()? {
        b'`' => '`',
        b'~' => '~',
        _ => return None,
    };
    let len = stripped
        .chars()
        .take_while(|candidate| *candidate == ch)
        .count();
    if len >= 3 {
        Some(FenceMarker {
            ch,
            len,
            blank_tail: stripped.get(len..).unwrap_or_default().trim().is_empty(),
        })
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
    if line.get(trimmed.len()..).unwrap_or_default() == "  " {
        out.push_str("  ");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{format_markdown, format_yaml};

    // -- YAML ---------------------------------------------------------------

    #[test]
    fn yaml_normalizes_crlf_and_adds_final_newline() {
        let result = format_yaml("a: 1\r\nb: 2");
        assert_eq!(result.code.as_str(), "a: 1\nb: 2\n");
        assert!(result.changed);
    }

    #[test]
    fn yaml_preserves_block_scalar_trailing_whitespace() {
        // Trailing spaces inside a literal block scalar are significant; YAML must
        // never touch line content.
        let source = "key: |\n  line with spaces   \n  next\n";
        let result = format_yaml(source);
        assert_eq!(result.code.as_str(), source);
        assert!(!result.changed);
    }

    #[test]
    fn yaml_is_idempotent_and_noop_when_clean() {
        let source = "# comment\nname: vize\nlist:\n  - a\n  - b\n";
        let first = format_yaml(source);
        assert_eq!(first.code.as_str(), source);
        let second = format_yaml(first.code.as_str());
        assert_eq!(first.code, second.code);
    }

    #[test]
    fn yaml_empty_stays_empty() {
        assert_eq!(format_yaml("").code.as_str(), "");
    }

    // -- Markdown -----------------------------------------------------------

    #[test]
    fn markdown_trims_trailing_whitespace() {
        let result = format_markdown("# Title   \n\nText with space \n");
        assert_eq!(result.code.as_str(), "# Title\n\nText with space\n");
    }

    #[test]
    fn markdown_preserves_two_space_hard_break() {
        let source = "line one  \nline two\n";
        let result = format_markdown(source);
        assert_eq!(result.code.as_str(), source);
        assert!(!result.changed);
    }

    #[test]
    fn markdown_preserves_fenced_code_block_verbatim() {
        let source = "```sh\necho hi   \n```\ntext   \n";
        let result = format_markdown(source);
        // Trailing spaces inside the fence are kept; outside is trimmed.
        assert_eq!(result.code.as_str(), "```sh\necho hi   \n```\ntext\n");
    }

    #[test]
    fn markdown_preserves_shorter_fence_inside_longer_fence() {
        let source = "````md\n```ts\nconst value = 1   \n```\n````\ntext   \n";
        let result = format_markdown(source);
        assert_eq!(
            result.code.as_str(),
            "````md\n```ts\nconst value = 1   \n```\n````\ntext\n"
        );
    }

    #[test]
    fn markdown_preserves_info_string_fence_inside_same_length_fence() {
        let source = "```md\n```ts\nconst value = 1   \nexample done   \n```\ntext   \n";
        let result = format_markdown(source);
        assert_eq!(
            result.code.as_str(),
            "```md\n```ts\nconst value = 1   \nexample done   \n```\ntext\n"
        );
    }

    #[test]
    fn markdown_preserves_indented_code_block() {
        let source = "para\n\n    code with spaces   \n";
        let result = format_markdown(source);
        assert_eq!(result.code.as_str(), source);
    }

    #[test]
    fn markdown_adds_final_newline_and_is_idempotent() {
        let first = format_markdown("# Title");
        assert_eq!(first.code.as_str(), "# Title\n");
        let second = format_markdown(first.code.as_str());
        assert_eq!(first.code, second.code);
    }
}
