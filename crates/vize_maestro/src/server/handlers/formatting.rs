//! Document, range and on-type formatting request routing.

use super::{
    DocumentFormattingParams, DocumentOnTypeFormattingParams, DocumentRangeFormattingParams,
    MaestroServer, Result, TextEdit,
};

pub(super) async fn formatting(
    server: &MaestroServer,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    if !server.state.lsp_features().formatting {
        return Ok(None);
    }

    let uri = &params.text_document.uri;

    // Standalone (petite-vue) HTML documents are not SFCs: running the SFC
    // formatter over them corrupts the file. Skip until a dedicated HTML
    // formatter lands (#1393).
    if crate::utils::is_standalone_html_path(uri.path()) {
        return Ok(None);
    }

    let Some(_content) = server.state.documents.text(uri) else {
        return Ok(None);
    };
    #[cfg(feature = "glyph")]
    {
        let (options, preserve) = server.state.get_formatter_context();
        Ok(
            super::super::format::format_document_with_template_whitespace(
                &_content,
                &options,
                server.state.type_checker_vue_version(),
                preserve,
            ),
        )
    }
    #[cfg(not(feature = "glyph"))]
    Ok(None)
}

pub(super) async fn range_formatting(
    server: &MaestroServer,
    params: DocumentRangeFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    if !server.state.lsp_features().formatting {
        return Ok(None);
    }

    let uri = &params.text_document.uri;
    let _range = params.range;

    // See `formatting`: standalone HTML must not go through the SFC formatter.
    if crate::utils::is_standalone_html_path(uri.path()) {
        return Ok(None);
    }

    let Some(_content) = server.state.documents.text(uri) else {
        return Ok(None);
    };
    #[cfg(feature = "glyph")]
    {
        let (options, preserve) = server.state.get_formatter_context();
        let path = uri.path();
        Ok(super::super::format::format_range_with_template_whitespace(
            &_content,
            path,
            _range,
            &options,
            server.state.type_checker_vue_version(),
            preserve,
        ))
    }
    #[cfg(not(feature = "glyph"))]
    Ok(None)
}

pub(super) async fn on_type_formatting(
    server: &MaestroServer,
    params: DocumentOnTypeFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    if !server.state.lsp_features().formatting {
        return Ok(None);
    }

    let uri = &params.text_document_position.text_document.uri;

    // See `formatting`: standalone HTML must not go through the SFC formatter.
    if crate::utils::is_standalone_html_path(uri.path()) {
        return Ok(None);
    }

    let Some(_content) = server.state.documents.text(uri) else {
        return Ok(None);
    };
    #[cfg(feature = "glyph")]
    {
        let (options, preserve) = server.state.get_formatter_context();
        let position = params.text_document_position.position;
        let path = uri.path();
        Ok(
            super::super::format::format_on_type_with_template_whitespace(
                &_content,
                path,
                position,
                &options,
                server.state.type_checker_vue_version(),
                preserve,
            ),
        )
    }
    #[cfg(not(feature = "glyph"))]
    Ok(None)
}
