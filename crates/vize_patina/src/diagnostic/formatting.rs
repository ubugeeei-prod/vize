//! Help text formatting and markdown rendering.
//!
//! Provides functions for rendering markdown help text into different
//! output formats: ANSI terminal codes, plain text, or raw markdown passthrough.

use vize_l0::String;
use vize_l0::ToCompactString;

// ANSI escape codes
const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_BOLD_OFF: &str = "\x1b[22m";
const ANSI_UNDERLINE: &str = "\x1b[4m";
const ANSI_UNDERLINE_OFF: &str = "\x1b[24m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_CYAN_OFF: &str = "\x1b[39m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_DIM_OFF: &str = "\x1b[22m";

/// Render target for help text conversion at output boundaries.
///
/// Help text is stored as raw markdown in `LintDiagnostic.help`.
/// Use this enum with [`render_help`] to convert at the output boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpRenderTarget {
    /// CLI/TUI: Convert markdown to ANSI escape codes.
    Ansi,
    /// LSP, JSON: Strip markdown syntax for clean text.
    PlainText,
    /// WASM/Playground: Pass through as-is (raw markdown).
    Markdown,
}

/// Render help text (raw markdown) for the given target.
///
/// This function should be called at output boundaries, not when creating diagnostics.
pub fn render_help(markdown: &str, target: HelpRenderTarget) -> String {
    match target {
        HelpRenderTarget::Ansi => render_markdown_to_ansi(markdown),
        HelpRenderTarget::PlainText => strip_markdown(markdown),
        HelpRenderTarget::Markdown => markdown.to_compact_string(),
    }
}

/// Convert markdown text to ANSI-formatted text for TUI display.
///
/// Supports:
/// - `**bold**` / `__bold__` -> ANSI bold
/// - `` `code` `` -> cyan
/// - ```` ``` ```` code blocks -> indented + dim
/// - `# Header` -> bold + underline
pub(crate) fn render_markdown_to_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 64);
    let mut in_code_block = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Handle code fence blocks (check before pushing newline to avoid extra \n)
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if !result.is_empty() {
            result.push('\n');
        }

        // Inside code block: indent + dim
        if in_code_block {
            result.push_str(ANSI_DIM);
            result.push_str("  ");
            result.push_str(line);
            result.push_str(ANSI_DIM_OFF);
            continue;
        }

        // Handle headers: # Header -> bold + underline
        if let Some(header_content) = trimmed.strip_prefix("# ") {
            result.push_str(ANSI_BOLD);
            result.push_str(ANSI_UNDERLINE);
            result.push_str(header_content);
            result.push_str(ANSI_UNDERLINE_OFF);
            result.push_str(ANSI_BOLD_OFF);
            continue;
        }
        if let Some(header_content) = trimmed.strip_prefix("## ") {
            result.push_str(ANSI_BOLD);
            result.push_str(ANSI_UNDERLINE);
            result.push_str(header_content);
            result.push_str(ANSI_UNDERLINE_OFF);
            result.push_str(ANSI_BOLD_OFF);
            continue;
        }
        if let Some(header_content) = trimmed.strip_prefix("### ") {
            result.push_str(ANSI_BOLD);
            result.push_str(header_content);
            result.push_str(ANSI_BOLD_OFF);
            continue;
        }

        // Process inline formatting
        render_inline_markdown(&mut result, line);
    }

    result
}

/// Process inline markdown formatting: **bold**, __bold__, `code`.
fn render_inline_markdown(out: &mut String, line: &str) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while let Some(&byte) = bytes.get(i) {
        // Inline code: `code`
        if byte == b'`'
            && let Some(end) = find_closing_backtick(bytes, i + 1)
        {
            out.push_str(ANSI_CYAN);
            out.push_str(line.get(i + 1..end).unwrap_or_default());
            out.push_str(ANSI_CYAN_OFF);
            i = end + 1;
            continue;
        }

        // Bold: **text** or __text__
        if bytes.get(i..i + 2) == Some(b"**".as_slice())
            && let Some(end) = find_closing_double(bytes, i + 2, b'*')
        {
            out.push_str(ANSI_BOLD);
            // Recursively process inline content within bold
            render_inline_markdown(out, line.get(i + 2..end).unwrap_or_default());
            out.push_str(ANSI_BOLD_OFF);
            i = end + 2;
            continue;
        }
        if bytes.get(i..i + 2) == Some(b"__".as_slice())
            && let Some(end) = find_closing_double(bytes, i + 2, b'_')
        {
            out.push_str(ANSI_BOLD);
            render_inline_markdown(out, line.get(i + 2..end).unwrap_or_default());
            out.push_str(ANSI_BOLD_OFF);
            i = end + 2;
            continue;
        }

        // Copy plain text up to the next delimiter byte. Delimiters are ASCII,
        // so both ends stay on char boundaries and non-ASCII text survives.
        let next = bytes
            .get(i + 1..)
            .and_then(|rest| rest.iter().position(|b| matches!(b, b'`' | b'*' | b'_')))
            .map_or(len, |offset| i + 1 + offset);
        out.push_str(line.get(i..next).unwrap_or_default());
        i = next;
    }
}

/// Find closing backtick for inline code.
fn find_closing_backtick(bytes: &[u8], start: usize) -> Option<usize> {
    let offset = bytes.get(start..)?.iter().position(|&b| b == b'`')?;
    Some(start + offset)
}

/// Find closing double delimiter (** or __).
fn find_closing_double(bytes: &[u8], start: usize, ch: u8) -> Option<usize> {
    let mut i = start;
    while i + 1 < bytes.len() {
        if bytes.get(i..i + 2) == Some([ch, ch].as_slice()) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Strip all markdown formatting from text, producing clean plain text.
pub(crate) fn strip_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_code_block = false;
    // Two scratch lines reused for the marker passes below.
    let (mut once, mut twice) = (String::default(), String::default());

    for line in text.lines() {
        let trimmed = line.trim();

        // Handle code fence blocks
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Inside code block: keep content with indent
        if in_code_block {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("  ");
            result.push_str(trimmed);
            continue;
        }

        // Skip empty lines (but preserve paragraph breaks)
        if trimmed.is_empty() {
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            continue;
        }

        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }

        // Strip markdown headers
        let content = trimmed
            .strip_prefix("### ")
            .or_else(|| trimmed.strip_prefix("## "))
            .or_else(|| trimmed.strip_prefix("# "))
            .unwrap_or(trimmed);

        // Strip bold/italic markers and inline code backticks, pass by pass
        // as `replace` would (removing `**` can join a `__`).
        if !content.contains(['*', '_', '`']) {
            result.push_str(content.trim());
            continue;
        }
        remove_into(content, "**", &mut once);
        remove_into(&once, "__", &mut twice);
        remove_into(&twice, "`", &mut once);
        result.push_str(once.trim());
    }

    // Trim trailing whitespace/newlines
    let kept = result.trim_end().len();
    result.truncate(kept);
    result
}

/// `source.replace(marker, "")` into a reused buffer.
fn remove_into(source: &str, marker: &str, out: &mut String) {
    out.clear();
    for part in source.split(marker) {
        out.push_str(part);
    }
}

#[cfg(test)]
mod strip_markdown_tests {
    use super::strip_markdown;
    use vize_l0::ToCompactString;

    /// The pass-by-pass `replace` chain `strip_markdown` used to allocate.
    fn by_replace(text: &str) -> vize_l0::String {
        let mut result = vize_l0::String::default();
        let mut in_code_block = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str("  ");
                result.push_str(trimmed);
                continue;
            }
            if trimmed.is_empty() {
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                continue;
            }
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            let content = trimmed
                .strip_prefix("### ")
                .or_else(|| trimmed.strip_prefix("## "))
                .or_else(|| trimmed.strip_prefix("# "))
                .unwrap_or(trimmed);
            let content = content.replace("**", "").replace("__", "").replace('`', "");
            result.push_str(content.trim());
        }
        result.trim_end().to_compact_string()
    }

    #[test]
    fn matches_the_replace_chain() {
        for text in [
            "",
            "plain help",
            "**Why:** keys help `v-for` track items.",
            "_**_ joined markers",
            "__**bold**__ and `` `nested` ``",
            "** spaced ** `  code  `",
            "# Title\n\n## Sub **x**\n### `three`",
            "#  Title with two spaces",
            "```vue\n<li v-for=\"i in items\" :key=\"i.id\">\n```\n**Fix:** add a key",
            "a*b_c `d`\n\n\n  trailing  \n\n",
            "\u{1F600} **emoji** __ä__",
        ] {
            assert_eq!(strip_markdown(text), by_replace(text), "{text:?}");
        }
    }
}
