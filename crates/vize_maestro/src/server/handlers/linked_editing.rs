//! Standard linked-editing request routing.

use super::{
    LinkedEditingRangeParams, LinkedEditingRanges, MaestroServer, Result, position_to_offset,
};

pub(super) async fn linked_editing_range(
    server: &MaestroServer,
    params: LinkedEditingRangeParams,
) -> Result<Option<LinkedEditingRanges>> {
    if !server.state.lsp_features().rename {
        return Ok(None);
    }

    #[cfg(feature = "experimental-source-navigation")]
    if server.state.native_linked_editing_enabled() {
        return server.native_linked_editing(params).await;
    }

    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    let Some(content) = server.state.documents.text(uri) else {
        return Ok(None);
    };
    let Some(offset) = position_to_offset(&content, position.line, position.character) else {
        return Ok(None);
    };

    Ok(crate::ide::linked_editing::LinkedEditingService::ranges(
        &content,
        uri.path(),
        offset,
    ))
}
