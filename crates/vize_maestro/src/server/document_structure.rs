//! Document-structure providers: folding ranges and selection ranges.
//!
//! Both answer "what are the syntactic regions of this authored document?", are
//! pure functions of the SFC block layout plus the template markup, and are
//! gated on the same `folding_ranges` feature flag. Keeping them together (and
//! out of `handlers.rs`, which is already over the per-file length budget)
//! makes that shared contract explicit.
#![allow(clippy::disallowed_methods)]

mod symbols;

pub(super) use symbols::document_symbols;

use tower_lsp::lsp_types::{
    FoldingRange, FoldingRangeKind, FoldingRangeParams, SelectionRange, SelectionRangeParams,
};

use super::ServerState;
use crate::ide::{SelectionRangeService, position_to_offset};

/// `textDocument/foldingRange`: one region per SFC block that spans more than a
/// single line.
pub(super) fn folding_ranges(
    state: &ServerState,
    params: &FoldingRangeParams,
) -> Option<Vec<FoldingRange>> {
    let uri = &params.text_document.uri;
    let content = state.documents.text(uri)?;

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: uri.path().to_string().into(),
        ..Default::default()
    };
    let descriptor = vize_atelier_sfc::parse_sfc(&content, options).ok()?;

    let mut ranges = Vec::new();
    let labelled_blocks = descriptor
        .template
        .as_ref()
        .map(|block| (&block.loc, "template"))
        .into_iter()
        .chain(
            descriptor
                .script_setup
                .as_ref()
                .map(|block| (&block.loc, "script setup")),
        )
        .chain(
            descriptor
                .script
                .as_ref()
                .map(|block| (&block.loc, "script")),
        )
        .chain(descriptor.styles.iter().map(|block| (&block.loc, "style")));

    for (loc, label) in labelled_blocks {
        if loc.start_line >= loc.end_line {
            continue;
        }
        ranges.push(FoldingRange {
            start_line: loc.start_line.saturating_sub(1) as u32,
            start_character: None,
            end_line: loc.end_line.saturating_sub(1) as u32,
            end_character: None,
            kind: Some(FoldingRangeKind::Region),
            collapsed_text: Some(label.to_string()),
        });
    }

    (!ranges.is_empty()).then_some(ranges)
}

/// `textDocument/selectionRange`: one expand/shrink chain per requested
/// position.
///
/// The LSP contract requires one chain per position, so a single unresolvable
/// position collapses the whole response to `None` rather than returning a
/// short array the client would mis-index.
pub(super) fn selection_ranges(
    state: &ServerState,
    params: &SelectionRangeParams,
) -> Option<Vec<SelectionRange>> {
    let uri = &params.text_document.uri;
    let content = state.documents.text(uri)?;
    let filename = uri.path().to_string();

    let mut ranges = Vec::with_capacity(params.positions.len());
    for position in &params.positions {
        let offset = position_to_offset(&content, position.line, position.character)?;
        ranges.push(SelectionRangeService::selection_range(
            &content, &filename, offset,
        )?);
    }

    (!ranges.is_empty()).then_some(ranges)
}

#[cfg(test)]
mod tests;
