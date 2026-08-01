//! `textDocument/onTypeFormatting`: re-indent the line being typed on.
//!
//! # What this does, and deliberately does not, do
//!
//! On-type formatting fires under the caret — on every `;`, `}` and newline the
//! user types (#3456). Anything that rewrites *content* there fights the
//! typist: it moves the caret, undoes a half-finished edit, and turns one
//! keystroke into an unbounded diff. So this handler only ever changes a line's
//! **leading whitespace**, and only on the line the request names.
//!
//! # Why the indent comes from the whole-document format
//!
//! The indent is not guessed from brace depth — it is read off the same
//! `vize fmt` run that backs Format Document, then applied to one line. Format
//! Document, Format Selection and Format On Type therefore agree by
//! construction rather than by three implementations happening to match.
//!
//! # Why the pairing is per block
//!
//! Reading an indent off "formatted line N" requires authored line N and
//! formatted line N to be the same line. Across a whole `.vue` that almost
//! never holds: the formatter breaks `<p>hello</p>` onto three lines, so one
//! unformatted element in the template shifts every line below it and the
//! script's indentation could never be answered.
//!
//! The pairing is therefore scoped to the SFC block the caret is in — the same
//! authored/formatted block pairing `format_range` projects through. A template
//! that is mid-edit no longer stops the script from being indented, and vice
//! versa.
//!
//! Within that block the handler still declines — returns no edits — whenever
//! the formatter would
//!
//! - add or remove lines, which would slide every later line out of its pair;
//!   or
//! - rewrite the line's own content, which is the formatter asking for an edit
//!   this request must not make.
//!
//! Declining is cheap and correct: the keystroke produces no edit, and Format
//! Document still does the full job on demand.

use tower_lsp::lsp_types::{Position, Range, TextEdit};

use super::blocks::block_spans;
use crate::ide::position_to_offset;

/// Re-indent `position.line` to the indent whole-document formatting gives it.
///
/// `None` means the request is not answerable at all: the document has no such
/// line, or the formatter could not parse the file — routine while typing.
/// `Some(vec![])` means "nothing to change here".
pub(crate) fn format_on_type(
    content: &str,
    filename: &str,
    position: Position,
    options: &vize_glyph::FormatOptions,
) -> Option<Vec<TextEdit>> {
    let line_start = position_to_offset(content, position.line, 0)?;

    let allocator = vize_glyph::Allocator::with_capacity(content.len());
    let formatted = vize_glyph::format_sfc_with_allocator(content, options, &allocator).ok()?;
    if !formatted.changed {
        return Some(Vec::new());
    }

    let authored = block_spans(content, filename)?;
    let target = block_spans(&formatted.code, filename)?;
    // A formatter must not add, drop or reorder blocks. If it did, the two
    // lists cannot be paired and there is no indent to read.
    if authored.len() != target.len() {
        return Some(Vec::new());
    }

    let Some(index) = authored
        .iter()
        .position(|&(start, end)| (start..=end).contains(&line_start))
    else {
        // Between blocks, or on a block's own tag line. Neither is content the
        // formatter indents.
        return Some(Vec::new());
    };

    let (authored_start, authored_end) = authored[index];
    let (target_start, target_end) = target[index];
    // Splitting on '\n' leaves the '\r' of a CRLF pair on the line. The
    // formatter writes the configured newline, which need not be the authored
    // one, so a surviving '\r' would make every CRLF line look rewritten and
    // silence the handler on CRLF documents.
    let authored_lines: Vec<&str> = content[authored_start..authored_end]
        .split('\n')
        .map(strip_cr)
        .collect();
    let target_lines: Vec<&str> = formatted.code[target_start..target_end]
        .split('\n')
        .map(strip_cr)
        .collect();
    // Line N of this block must still be line N after formatting.
    if authored_lines.len() != target_lines.len() {
        return Some(Vec::new());
    }

    // Block content opens partway through the tag's own line, so relative line
    // 0 is the tail of `<script …>` rather than a line of its own.
    let relative = content[authored_start..line_start].matches('\n').count();
    if relative == 0 {
        return Some(Vec::new());
    }
    let (Some(&authored_line), Some(&target_line)) =
        (authored_lines.get(relative), target_lines.get(relative))
    else {
        return Some(Vec::new());
    };

    let authored_indent = indent_of(authored_line);
    let target_indent = indent_of(target_line);
    // Everything after the indent must already match: this request re-indents,
    // it never rewrites what the user is in the middle of typing.
    if authored_line[authored_indent.len()..] != target_line[target_indent.len()..]
        || authored_indent == target_indent
    {
        return Some(Vec::new());
    }

    Some(vec![TextEdit {
        range: Range {
            start: Position::new(position.line, 0),
            // Indentation is spaces and tabs, one UTF-16 unit each, so the byte
            // length is also the character offset the client expects.
            end: Position::new(position.line, authored_indent.len() as u32),
        },
        #[allow(clippy::disallowed_methods)]
        new_text: target_indent.to_string(),
    }])
}

/// The line without the `'\r'` a CRLF pair leaves behind when splitting on
/// `'\n'`. Only the trailing one: a lone `'\r'` mid-line is authored content.
fn strip_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

/// The leading run of spaces and tabs. Deliberately not `trim_start`, which
/// also eats non-breaking spaces and other Unicode whitespace that is authored
/// content rather than indentation.
fn indent_of(line: &str) -> &str {
    let end = line
        .find(|ch: char| ch != ' ' && ch != '\t')
        .unwrap_or(line.len());
    &line[..end]
}

#[cfg(test)]
mod tests;
