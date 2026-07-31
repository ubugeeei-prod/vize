//! Inline annotations the editor renders over an authored document: code
//! lenses, inlay hints and colour swatches.
//!
//! All three answer "what should be drawn on top of this text?" from the
//! authored document alone — no type checking, no corsa — and all three are
//! per-document rather than per-position. Keeping them together (and out of
//! `handlers.rs`, which is already over the per-file length budget) makes that
//! shared contract explicit, the same way `document_structure` groups folding
//! and selection ranges.
#![allow(clippy::disallowed_methods)]

mod document_color;

use tower_lsp::lsp_types::{
    CodeLens, CodeLensParams, ColorInformation, ColorPresentation, ColorPresentationParams,
    DocumentColorParams, InlayHint, InlayHintParams,
};

use document_color::DocumentColorService;

use super::ServerState;
use crate::ide::{CodeLensService, InlayHintService};

pub(super) fn code_lens(state: &ServerState, params: &CodeLensParams) -> Option<Vec<CodeLens>> {
    let uri = &params.text_document.uri;
    let content = state.documents.text(uri)?;
    let lenses = CodeLensService::get_lenses(&content, uri);
    (!lenses.is_empty()).then_some(lenses)
}

pub(super) fn inlay_hint(state: &ServerState, params: &InlayHintParams) -> Option<Vec<InlayHint>> {
    let uri = &params.text_document.uri;
    let content = state.documents.text(uri)?;
    let hints = InlayHintService::get_hints_with_ecosystem(
        &content,
        uri,
        params.range,
        state.lsp_features().ecosystem,
    );
    (!hints.is_empty()).then_some(hints)
}

/// `textDocument/documentColor` returns an array, not `null`: an empty array is
/// the correct answer for a document with no colour literals, and the LSP
/// contract has no "unknown" here.
pub(super) fn document_color(
    state: &ServerState,
    params: &DocumentColorParams,
) -> Vec<ColorInformation> {
    let uri = &params.text_document.uri;
    let Some(content) = state.documents.text(uri) else {
        return Vec::new();
    };
    DocumentColorService::colors(&content, uri.path())
}

/// `textDocument/colorPresentation` is a pure function of the colour the picker
/// produced: the document is not consulted, because the client replaces the
/// range it already sent.
pub(super) fn color_presentation(params: &ColorPresentationParams) -> Vec<ColorPresentation> {
    DocumentColorService::presentations(params.color)
}

#[cfg(test)]
mod tests;
