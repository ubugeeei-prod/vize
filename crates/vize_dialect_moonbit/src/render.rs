//! Human rendering of a checked SFC: `moonc`'s messages at their
//! authored positions, with the authored line underlined.
//!
//! The output is deterministic (no colors, no absolute paths, columns in
//! Unicode scalar values), so the fixtures pin it byte for byte and the
//! showcase prints exactly what CI checked.

use vize_s0::{String, append};

use crate::diagnostic::{Mapped, Origin};
use crate::lines::Lines;
use crate::projection::Projection;

/// Render every mapped diagnostic and every unsupported position of one
/// SFC named `file_name`.
#[must_use]
pub fn render(
    source: &str,
    file_name: &str,
    projection: &Projection<'_>,
    diagnostics: &[Mapped],
) -> String {
    let lines = Lines::new(source);
    let generated_lines = Lines::new(&projection.text);
    let mut out = String::default();
    for mapped in diagnostics {
        let diagnostic = &mapped.diagnostic;
        append!(
            out,
            "{}[{}]: {}\n",
            diagnostic.level.as_str(),
            diagnostic.code,
            diagnostic.message
        );
        match (mapped.origin, mapped.span) {
            (Origin::Projection, _) | (_, None) => {
                let at = diagnostic.start;
                append!(
                    out,
                    "  --> {}:{}:{} (generated projection)\n",
                    projection.file_name,
                    at.line,
                    at.col
                );
                excerpt(
                    &mut out,
                    &generated_lines,
                    at.line,
                    at.col,
                    diagnostic.end.col,
                );
            }
            (_, Some(span)) => {
                let start = lines.position(span.start as usize);
                let end = lines.position(span.end as usize);
                append!(out, "  --> {file_name}:{}:{}\n", start.line, start.col);
                let last = if end.line == start.line {
                    end.col
                } else {
                    let width = lines.line_text(start.line).chars().count();
                    u32::try_from(width + 1).unwrap_or(u32::MAX)
                };
                excerpt(&mut out, &lines, start.line, start.col, last);
            }
        }
        out.push('\n');
    }
    for unsupported in &projection.unsupported {
        let at = lines.position(unsupported.span.start as usize);
        append!(
            out,
            "note: `{}` is outside the P6-4a projection subset and was not checked\n  --> {file_name}:{}:{}\n\n",
            unsupported.what,
            at.line,
            at.col
        );
    }
    // Blocks are blank-line separated; the text ends with one newline.
    if out.ends_with("\n\n") {
        out.pop();
    }
    out
}

/// One source line with carets under columns `from..to` (at least one).
fn excerpt(mut out: &mut String, lines: &Lines<'_>, line: u32, from: u32, to: u32) {
    let text = lines.line_text(line);
    let gutter = line
        .checked_ilog10()
        .map_or(1, |digits| digits as usize + 1);
    append!(out, "{:gutter$} |\n", "");
    append!(out, "{line} | {text}\n");
    let pad = from.saturating_sub(1) as usize;
    append!(out, "{:gutter$} | {:pad$}", "", "");
    for _ in 0..to.saturating_sub(from).max(1) {
        out.push('^');
    }
    out.push('\n');
}
