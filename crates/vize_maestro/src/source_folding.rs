//! Experimental comment folding from one authentic native Program observation.
//!
//! This projection reads the retained native parse. It does not parse again or
//! select a production LSP route. SFC scripts need their producer-owned full-file
//! bridge before this standalone provider can project into a component document.

use oxc_ast::ast::CommentKind;
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};
use vize_l0::line_index::LineBreaks;
use vize_l1::embed::syntax::NativeSyntax;

/// A refusal preserves the caller's original observation for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFoldingRefusal {
    UnadmittedProgram,
    UnsupportedSourceCoordinates,
    InvalidAuthoredComment,
}

/// Project retained multiline block comments in their original source order.
///
/// Only genuine admitted zero-origin Programs are accepted. Wrapped expressions,
/// recovered/Flow syntax and decoded or nonzero-origin embeds stay refused.
/// These coordinate checks do not attest a sealed File or an on-disk document.
/// Ranges hide the comment's interior and retain its closing delimiter line;
/// adjacent-line and single-line comments therefore produce no range.
pub fn program_comment_ranges(
    syntax: &NativeSyntax<'_>,
) -> Result<Vec<FoldingRange>, SourceFoldingRefusal> {
    if syntax.admitted_program().is_none() {
        return Err(SourceFoldingRefusal::UnadmittedProgram);
    }
    let source = syntax.source();
    let text = source.text();
    let span = source.span();
    if source.decode_map().is_some() || span.start != 0 || span.end as usize != text.len() {
        return Err(SourceFoldingRefusal::UnsupportedSourceCoordinates);
    }
    // LSP counts CR, LF and CRLF, while LS/PS stay inside the authored line.
    // Build this table once; each retained comment uses two binary lookups.
    let lines: Vec<_> = LineBreaks::Lsp.line_starts(text).collect();
    let line_at = |offset: usize| -> u32 {
        lines
            .partition_point(|start| *start <= offset)
            .saturating_sub(1) as u32
    };
    let mut ranges = Vec::new();
    for comment in syntax.comments() {
        if !matches!(
            comment.kind(),
            CommentKind::SingleLineBlock | CommentKind::MultiLineBlock
        ) {
            continue;
        }
        let span = comment
            .authored_span()
            .map_err(|_| SourceFoldingRefusal::InvalidAuthoredComment)?;
        let bytes = text
            .get(span.start as usize..span.end as usize)
            .ok_or(SourceFoldingRefusal::InvalidAuthoredComment)?;
        if !bytes.starts_with("/*") || !bytes.ends_with("*/") {
            return Err(SourceFoldingRefusal::InvalidAuthoredComment);
        }
        let start_line = line_at(span.start as usize);
        let close_line = line_at(span.end as usize - 2);
        if close_line.saturating_sub(start_line) < 2 {
            continue;
        }
        ranges.push(FoldingRange {
            start_line,
            start_character: None,
            end_line: close_line - 1,
            end_character: None,
            kind: Some(FoldingRangeKind::Comment),
            collapsed_text: None,
        });
    }
    Ok(ranges)
}

#[cfg(test)]
mod tests;
