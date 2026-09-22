use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{Location, ReferenceParams},
};

use crate::ide::{IdeContext, ReferencesService, position_to_offset};
use crate::server::MaestroServer;

pub(super) async fn references(
    server: &MaestroServer,
    params: ReferenceParams,
) -> Result<Option<Vec<Location>>> {
    if !server.state.lsp_features().references {
        return Ok(None);
    }
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let include_declaration = params.context.include_declaration;
    let Some(content) = server.state.documents.text(uri) else {
        return Ok(None);
    };
    let Some(offset) = position_to_offset(&content, position.line, position.character) else {
        return Ok(None);
    };
    let ctx = IdeContext::new(&server.state, uri, offset).expect("document is open");

    #[cfg(feature = "native")]
    if crate::utils::is_jsx_path(uri.path()) {
        if server.state.jsx_typecheck_enabled() {
            return Ok(crate::ide::JsxReferencesService::references(
                &ctx,
                include_declaration,
                server.state.get_corsa_bridge().await,
            )
            .await);
        }
        return Ok(None);
    }
    #[cfg(feature = "native")]
    {
        Ok(ReferencesService::references_with_corsa(
            &ctx,
            include_declaration,
            server.state.get_corsa_bridge().await,
        )
        .await)
    }
    #[cfg(not(feature = "native"))]
    Ok(ReferencesService::references(&ctx, include_declaration))
}
