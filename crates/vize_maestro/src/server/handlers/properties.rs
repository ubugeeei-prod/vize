//! Authored-document property requests retain their own project settings.

use super::MaestroServer;
use crate::ide::{DocumentHighlightService, DocumentLinkService, IdeContext, position_to_offset};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CodeLens, CodeLensParams, ColorInformation, ColorPresentation, ColorPresentationParams,
        DocumentColorParams, DocumentHighlight, DocumentHighlightParams, DocumentLink,
        DocumentLinkParams, DocumentSymbolParams, DocumentSymbolResponse, FoldingRange,
        FoldingRangeParams, SelectionRange, SelectionRangeParams, SemanticTokensParams,
        SemanticTokensRangeParams, SemanticTokensRangeResult, SemanticTokensResult,
    },
};

impl MaestroServer {
    pub(super) async fn document_highlight_request(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        if !server.state.lsp_features().references {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(content) = server.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(ctx) = position_to_offset(&content, position.line, position.character)
            .and_then(|offset| IdeContext::new(&server.state, uri, offset))
        else {
            return Ok(None);
        };

        Ok(DocumentHighlightService::highlights(&ctx))
    }

    pub(super) async fn document_symbol_request(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let server = self.for_document(&params.text_document.uri).await;
        if !server.state.lsp_features().document_symbols {
            return Ok(None);
        }
        Ok(super::super::document_structure::document_symbols(
            &server.state,
            &params,
        ))
    }

    pub(super) async fn semantic_tokens_full_request(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let server = self.for_document(&params.text_document.uri).await;
        Ok(super::super::semantic_tokens::full(&server.state, &params))
    }

    pub(super) async fn semantic_tokens_range_request(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let server = self.for_document(&params.text_document.uri).await;
        Ok(super::super::semantic_tokens::range(&server.state, &params))
    }

    pub(super) async fn code_lens_request(
        &self,
        params: CodeLensParams,
    ) -> Result<Option<Vec<CodeLens>>> {
        let server = self.for_document(&params.text_document.uri).await;
        if !server.state.lsp_features().code_lens {
            return Ok(None);
        }
        Ok(super::super::annotations::code_lens(&server.state, &params))
    }

    pub(super) async fn document_link_request(
        &self,
        params: DocumentLinkParams,
    ) -> Result<Option<Vec<DocumentLink>>> {
        let server = self.for_document(&params.text_document.uri).await;
        if !server.state.lsp_features().document_links {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        let Some(content) = server.state.documents.text(uri) else {
            return Ok(None);
        };
        let links = DocumentLinkService::get_links(&server.state, &content, uri);

        if links.is_empty() {
            Ok(None)
        } else {
            Ok(Some(links))
        }
    }

    pub(super) async fn document_color_request(
        &self,
        params: DocumentColorParams,
    ) -> Result<Vec<ColorInformation>> {
        let server = self.for_document(&params.text_document.uri).await;
        if !server.state.lsp_features().document_links {
            return Ok(Vec::new());
        }
        Ok(super::super::annotations::document_color(
            &server.state,
            &params,
        ))
    }

    pub(super) async fn color_presentation_request(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        let server = self.for_document(&params.text_document.uri).await;
        if !server.state.lsp_features().document_links {
            return Ok(Vec::new());
        }
        Ok(super::super::annotations::color_presentation(&params))
    }

    pub(super) async fn folding_range_request(
        &self,
        params: FoldingRangeParams,
    ) -> Result<Option<Vec<FoldingRange>>> {
        let server = self.for_document(&params.text_document.uri).await;
        if !server.state.lsp_features().folding_ranges {
            return Ok(None);
        }
        Ok(super::super::document_structure::folding_ranges(
            &server.state,
            &params,
        ))
    }

    pub(super) async fn selection_range_request(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        let server = self.for_document(&params.text_document.uri).await;
        if !server.state.lsp_features().folding_ranges {
            return Ok(None);
        }
        Ok(super::super::document_structure::selection_ranges(
            &server.state,
            &params,
        ))
    }
}
