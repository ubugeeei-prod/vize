//! Document-structure providers: folding ranges and selection ranges.
//!
//! Both answer "what are the syntactic regions of this authored document?", are
//! pure functions of the SFC block layout plus the template markup, and are
//! gated on the same `folding_ranges` feature flag. Keeping them together (and
//! out of `handlers.rs`, which is already over the per-file length budget)
//! makes that shared contract explicit.
#![allow(clippy::disallowed_methods)]

mod folding;
mod symbols;

pub(super) use folding::folding_ranges;
pub(super) use symbols::document_symbols;

use tower_lsp::lsp_types::{SelectionRange, SelectionRangeParams};

use super::ServerState;
use crate::ide::{SelectionRangeService, position_to_offset};

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
