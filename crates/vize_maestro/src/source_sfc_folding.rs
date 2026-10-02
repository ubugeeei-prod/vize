//! Comment projection from the producer's genuinely admitted whole SFC/File.
//! This adapter reads the retained Programs and never parses or assembles a File.

use oxc_ast::ast::CommentKind;
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};
use vize_l0::line_index::LineBreaks;
use vize_l1_to_l2::native_file::NativeSfc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfcFoldingRefusal {
    MissingRetainedProgram,
    InvalidAuthoredComment,
}

/// Project original script comments onto this exact producer-owned whole File.
///
/// The input is the private-constructor NativeSfc admission view, not a caller
/// snapshot, standalone Program or diagnostic-only SFC observation. Its producer
/// already checks complete File membership, profiles, roles and physical source
/// identity. Unsupported or refused observations cannot enter this function.
/// This projection does not select the production LSP route or attest whole
/// historical response parity. CR/LF/CRLF follow LSP; LS/PS remain within a line.
pub fn sfc_comment_ranges(
    native: &NativeSfc<'_, '_>,
) -> Result<Vec<FoldingRange>, SfcFoldingRefusal> {
    let source = native.file().file().artifact().source();
    let lines: Vec<_> = LineBreaks::Lsp.line_starts(source).collect();
    let line_at = |offset: usize| -> u32 {
        lines
            .partition_point(|start| *start <= offset)
            .saturating_sub(1) as u32
    };
    let mut ranges = Vec::new();
    for script in native.observation().scripts() {
        let syntax = script
            .syntax()
            .ok_or(SfcFoldingRefusal::MissingRetainedProgram)?;
        let block = script.block().span();
        for comment in syntax.comments() {
            if !matches!(
                comment.kind(),
                CommentKind::SingleLineBlock | CommentKind::MultiLineBlock
            ) {
                continue;
            }
            // The retained producer-owned comment maps through its actual
            // authored source; no guessed offset or equal-string reparse.
            let span = comment
                .authored_span()
                .map_err(|_| SfcFoldingRefusal::InvalidAuthoredComment)?;
            if span.start < block.start || span.end > block.end {
                return Err(SfcFoldingRefusal::InvalidAuthoredComment);
            }
            let bytes = source
                .get(span.start as usize..span.end as usize)
                .ok_or(SfcFoldingRefusal::InvalidAuthoredComment)?;
            if !bytes.starts_with("/*") || !bytes.ends_with("*/") {
                return Err(SfcFoldingRefusal::InvalidAuthoredComment);
            }
            let start_line = line_at(span.start as usize);
            let close_line = line_at(span.end as usize - 2);
            if close_line.saturating_sub(start_line) < 2 {
                continue;
            }
            ranges.push((
                span.start,
                FoldingRange {
                    start_line,
                    start_character: None,
                    end_line: close_line - 1,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Comment),
                    collapsed_text: None,
                },
            ));
        }
    }
    // Producers observe ordinary/setup roles, which can differ from authored
    // order. Protocol ranges retain the original whole-document source order.
    ranges.sort_by_key(|(start, _)| *start);
    Ok(ranges.into_iter().map(|(_, range)| range).collect())
}

#[cfg(test)]
mod tests;
