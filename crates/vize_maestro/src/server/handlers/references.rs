use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{Location, ReferenceParams},
};

use crate::ide::{IdeContext, ReferencesService};
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
    let Some(ctx) = IdeContext::at_position(&server.state, uri, position) else {
        return Ok(None);
    };

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
    if crate::utils::is_plain_script_path(uri.path()) {
        return Ok(crate::ide::ScriptSymbolsService::references(
            &ctx,
            include_declaration,
            server.state.get_corsa_bridge().await,
        )
        .await);
    }
    #[cfg(feature = "native")]
    {
        let stamp = server.state.corsa_request_stamp();
        if let Some(response) =
            server
                .state
                .cached_native_references(uri, position, include_declaration, stamp)
        {
            return Ok(response);
        }
        let response = if let Some(locations) =
            ReferencesService::setup_local_references(&ctx, include_declaration)
        {
            Some(locations)
        } else {
            ReferencesService::project_references_with_corsa(
                &ctx,
                include_declaration,
                server.state.get_corsa_bridge().await,
            )
            .await
        };
        server.state.cache_native_references(
            uri,
            position,
            include_declaration,
            stamp,
            response.clone(),
        );
        Ok(response)
    }
    #[cfg(not(feature = "native"))]
    Ok(ReferencesService::references(&ctx, include_declaration))
}
