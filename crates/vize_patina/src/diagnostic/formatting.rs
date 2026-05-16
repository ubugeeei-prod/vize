//! Help text formatting and markdown rendering.
//!
//! Provides functions for rendering markdown help text into different
//! output formats: ANSI terminal codes, plain text, or raw markdown passthrough.

use vize_carton::String;
use vize_carton::ToCompactString;

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

/// Strip markdown formatting and return the first meaningful line.
pub(crate) fn strip_markdown_first_line(text: &str) -> String {
    let mut in_code_block = false;
    for line in text.lines() {
        let trimmed = line.trim();
        // Track code fence blocks
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        // Skip lines inside code blocks
        if in_code_block {
            continue;
        }
        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }
        // Strip markdown bold/italic
        let stripped = trimmed.replace("**", "").replace("__", "").replace('`', "");
        // Skip lines that are just markdown headers
        let stripped = stripped.trim_start_matches('#').trim();
        if stripped.is_empty() {
            continue;
        }
        return stripped.to_compact_string();
    }
    text.lines().next().unwrap_or(text).to_compact_string()
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

    while i < len {
        // Inline code: `code`
        if bytes[i] == b'`' {
            if let Some(end) = find_closing_backtick(bytes, i + 1) {
                out.push_str(ANSI_CYAN);
                out.push_str(&line[i + 1..end]);
                out.push_str(ANSI_CYAN_OFF);
                i = end + 1;
                continue;
            }
        }

        // Bold: **text** or __text__
        if i + 1 < len && bytes[i] == b'*' && bytes[i + 1] == b'*' {
            if let Some(end) = find_closing_double(bytes, i + 2, b'*') {
                out.push_str(ANSI_BOLD);
                // Recursively process inline content within bold
                render_inline_markdown(out, &line[i + 2..end]);
                out.push_str(ANSI_BOLD_OFF);
                i = end + 2;
                continue;
            }
        }
        if i + 1 < len && bytes[i] == b'_' && bytes[i + 1] == b'_' {
            if let Some(end) = find_closing_double(bytes, i + 2, b'_') {
                out.push_str(ANSI_BOLD);
                render_inline_markdown(out, &line[i + 2..end]);
                out.push_str(ANSI_BOLD_OFF);
                i = end + 2;
                continue;
            }
        }

        out.push(bytes[i] as char);
        i += 1;
    }
}

/// Find closing backtick for inline code.
fn find_closing_backtick(bytes: &[u8], start: usize) -> Option<usize> {
    (start..bytes.len()).find(|&i| bytes[i] == b'`')
}

/// Find closing double delimiter (** or __).
fn find_closing_double(bytes: &[u8], start: usize, ch: u8) -> Option<usize> {
    let mut i = start;
    while i + 1 < bytes.len() {
        if bytes[i] == ch && bytes[i + 1] == ch {
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

        // Strip bold/italic markers and inline code backticks
        let content = content.replace("**", "").replace("__", "").replace('`', "");
        result.push_str(content.trim());
    }

    // Trim trailing whitespace/newlines
    let trimmed = result.trim_end();
    trimmed.to_compact_string()
}
