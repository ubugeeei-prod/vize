//! `textDocument/semanticTokens` for an open document.
//!
//! Vue SFCs read the resident descriptor (P5-6c). `.jsx`/`.tsx` highlight
//! dynamic expressions directly; that path is structural and not gated on
//! `typeChecker.jsxTypecheck`. `.art.vue` files are tokenized inside the SFC
//! service and are not parsed.

use tower_lsp::lsp_types::{
    SemanticTokensParams, SemanticTokensRangeParams, SemanticTokensRangeResult,
    SemanticTokensResult,
};

use super::ServerState;
use crate::ide::SemanticTokensService;

pub(super) fn full(
    state: &ServerState,
    params: &SemanticTokensParams,
) -> Option<SemanticTokensResult> {
    if !state.lsp_features().semantic_tokens {
        return None;
    }
    let uri = &params.text_document.uri;
    let content = state.documents.text(uri)?;
    // `.jsx`/`.tsx`: highlight the dynamic JSX expressions.
    if crate::utils::is_jsx_path(uri.path()) {
        return crate::ide::JsxSemanticTokensService::tokens(&content, uri);
    }
    SemanticTokensService::get_tokens(state, &content, uri)
}

pub(super) fn range(
    state: &ServerState,
    params: &SemanticTokensRangeParams,
) -> Option<SemanticTokensRangeResult> {
    if !state.lsp_features().semantic_tokens {
        return None;
    }
    let uri = &params.text_document.uri;
    let content = state.documents.text(uri)?;
    if crate::utils::is_jsx_path(uri.path()) {
        return crate::ide::JsxSemanticTokensService::tokens_range(&content, uri, params.range);
    }
    SemanticTokensService::get_tokens_range(state, &content, uri, params.range)
}
