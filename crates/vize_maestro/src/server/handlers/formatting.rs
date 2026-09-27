//! Document, range and on-type formatting request routing.

use super::*;

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
        let options = server.state.get_format_options();
        return Ok(super::super::format::format_document(&_content, &options));
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
        let options = server.state.get_format_options();
        let path = uri.path();
        return Ok(super::super::format::format_range(
            &_content, path, _range, &options,
        ));
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
        let options = server.state.get_format_options();
        let position = params.text_document_position.position;
        let path = uri.path();
        return Ok(super::super::format::format_on_type(
            &_content, path, position, &options,
        ));
    }
    #[cfg(not(feature = "glyph"))]
    Ok(None)
}
