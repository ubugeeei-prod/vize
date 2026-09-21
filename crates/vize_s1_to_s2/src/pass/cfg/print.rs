//! The canonical text form of [`ComplexityFacts`]: the TS-17 snapshot body
//! and the breakdown humans read ("where does this component's complexity
//! come from").
//!
//! One header block of totals, then one row per contribution in breakdown
//! order: kind, authored span, nesting, both increments, and the first
//! line of the authored text the span covers (clipped).

use core::fmt::Write as _;

use vize_s0::String;

use super::{ComplexityFacts, Contribution};

/// Longest excerpt a row prints before clipping with `…`.
const EXCERPT_CHARS: usize = 48;

/// Print `facts` against the `source` its spans index.
#[must_use]
pub fn print_facts(facts: &ComplexityFacts, source: &str) -> String {
    let mut out = String::default();
    let _ = writeln!(out, "[complexity]");
    let _ = writeln!(out, "cyclomatic={}", facts.cyclomatic);
    let _ = writeln!(out, "cognitive={}", facts.cognitive);
    let _ = writeln!(out, "unknown={}", facts.unknown);
    let _ = writeln!(out, "max-nesting={}", facts.max_nesting);
    let _ = writeln!(out, "rows={}", facts.contributions.len());
    if facts.contributions.is_empty() {
        return out;
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "[complexity.breakdown]");
    for row in &facts.contributions {
        print_row(&mut out, row, source);
    }
    out
}

fn print_row(out: &mut String, row: &Contribution, source: &str) {
    let _ = write!(
        out,
        "{:<11} @{}:{} nesting={} cyclomatic+{} cognitive+{}",
        row.kind.as_str(),
        row.span.start,
        row.span.end,
        row.nesting,
        row.cyclomatic,
        row.cognitive,
    );
    let excerpt = row.span.slice(source);
    let first_line = excerpt.lines().next().unwrap_or("").trim();
    if !first_line.is_empty() {
        out.push_str("  ");
        push_clipped(out, first_line, excerpt.contains('\n'));
    }
    out.push('\n');
}

fn push_clipped(out: &mut String, text: &str, more_lines: bool) {
    let mut chars = text.char_indices();
    match chars.nth(EXCERPT_CHARS) {
        Some((cut, _)) => {
            out.push_str(&text[..cut]);
            out.push('…');
        }
        None => {
            out.push_str(text);
            if more_lines {
                out.push('…');
            }
        }
    }
}
