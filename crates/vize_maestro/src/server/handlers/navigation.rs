use tower_lsp::lsp_types::request::{
    GotoDeclarationParams, GotoDeclarationResponse, GotoImplementationParams,
    GotoImplementationResponse, GotoTypeDefinitionParams, GotoTypeDefinitionResponse,
};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Moniker, MonikerParams},
};

use super::super::MaestroServer;
#[cfg(feature = "native")]
use crate::ide::{
    DeclarationService, ImplementationService, JsxDeclarationService, JsxImplementationService,
    JsxService, JsxTypeDefinitionService, ScriptSymbolsService, TypeDefinitionService,
};
use crate::ide::{DefinitionService, IdeContext, position_to_offset};

pub(super) type ImplParams = GotoImplementationParams;
pub(super) type ImplResponse = GotoImplementationResponse;
pub(super) type DefParams = GotoDefinitionParams;
pub(super) type DefResponse = GotoDefinitionResponse;
pub(super) type TypeDefParams = GotoTypeDefinitionParams;
pub(super) type TypeDefResponse = GotoTypeDefinitionResponse;
pub(super) type DeclParams = GotoDeclarationParams;
pub(super) type DeclResponse = GotoDeclarationResponse;

/// Monikers of the symbol at a position, from the targets go-to-definition
/// already resolves (see [`crate::ide::moniker`]).
pub(super) async fn moniker(
    server: &MaestroServer,
    params: MonikerParams,
) -> Result<Option<Vec<Moniker>>> {
    let definition = goto_definition(
        server,
        GotoDefinitionParams {
            text_document_position_params: params.text_document_position_params,
            work_done_progress_params: params.work_done_progress_params,
            partial_result_params: params.partial_result_params,
        },
    )
    .await?;
    let locations = match definition {
        None => return Ok(None),
        Some(GotoDefinitionResponse::Scalar(location)) => vec![location],
        Some(GotoDefinitionResponse::Array(locations)) => locations,
        Some(GotoDefinitionResponse::Link(links)) => links
            .into_iter()
            .map(|link| Location::new(link.target_uri, link.target_selection_range))
            .collect(),
    };
    let mut monikers: Vec<Moniker> = Vec::new();
    for location in locations {
        let Ok(path) = location.uri.to_file_path() else {
            continue;
        };
        let Some(text) = server
            .state
            .documents
            .text(&location.uri)
            .or_else(|| std::fs::read_to_string(&path).ok())
        else {
            continue;
        };
        if let Some(moniker) =
            crate::ide::moniker::moniker_for_definition(&path, &text, location.range)
            && !monikers.contains(&moniker)
        {
            monikers.push(moniker);
        }
    }
    Ok((!monikers.is_empty()).then_some(monikers))
}

pub(super) async fn goto_definition(
    server: &MaestroServer,
    params: DefParams,
) -> Result<Option<DefResponse>> {
    if !server.state.lsp_features().definition {
        return Ok(None);
    }

    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let Some(content) = server.state.documents.text(uri) else {
        return Ok(None);
    };
    let Some(offset) = position_to_offset(&content, position.line, position.character) else {
        return Ok(None);
    };

    let Some(ctx) = IdeContext::new(&server.state, uri, offset) else {
        return Ok(None);
    };

    // Type-aware go-to-definition for `.jsx`/`.tsx` (opt-in
    // `typeChecker.jsxTypecheck`). React `.tsx` is untouched when off.
    #[cfg(feature = "native")]
    if crate::utils::is_jsx_path(uri.path()) {
        if server.state.jsx_typecheck_enabled() {
            let corsa_bridge = server.state.get_corsa_bridge().await;
            if let Some(response) = JsxService::definition(&ctx, corsa_bridge).await {
                return Ok(Some(response));
            }
        }
        return Ok(None);
    }

    #[cfg(feature = "native")]
    if crate::utils::is_plain_script_path(uri.path()) {
        return Ok(
            ScriptSymbolsService::definition(&ctx, server.state.get_corsa_bridge().await).await,
        );
    }

    #[cfg(feature = "native")]
    {
        let corsa_bridge = server.state.get_corsa_bridge().await;
        if let Some(response) = DefinitionService::definition_with_corsa(&ctx, corsa_bridge).await {
            return Ok(Some(response));
        }
    }

    #[cfg(not(feature = "native"))]
    if let Some(response) = DefinitionService::definition(&ctx) {
        return Ok(Some(response));
    }

    Ok(None)
}

pub(super) async fn goto_type_definition(
    server: &MaestroServer,
    params: TypeDefParams,
) -> Result<Option<TypeDefResponse>> {
    if !server.state.lsp_features().definition || !server.state.lsp_features().typecheck {
        return Ok(None);
    }

    #[cfg(feature = "native")]
    {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(content) = server.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let Some(ctx) = IdeContext::new(&server.state, uri, offset) else {
            return Ok(None);
        };

        if crate::utils::is_jsx_path(uri.path()) {
            if server.state.jsx_typecheck_enabled() {
                let corsa_bridge = server.state.get_corsa_bridge().await;
                return Ok(JsxTypeDefinitionService::type_definition(&ctx, corsa_bridge).await);
            }
            return Ok(None);
        }

        let corsa_bridge = server.state.get_corsa_bridge().await;
        return Ok(TypeDefinitionService::type_definition_with_corsa(&ctx, corsa_bridge).await);
    }

    #[cfg(not(feature = "native"))]
    {
        let _ = params;
        Ok(None)
    }
}

pub(super) async fn goto_declaration(
    server: &MaestroServer,
    params: DeclParams,
) -> Result<Option<DeclResponse>> {
    if !server.state.lsp_features().definition {
        return Ok(None);
    }

    #[cfg(feature = "native")]
    {
        if !server.state.lsp_features().typecheck {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(content) = server.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let Some(ctx) = IdeContext::new(&server.state, uri, offset) else {
            return Ok(None);
        };

        if crate::utils::is_jsx_path(uri.path()) {
            if server.state.jsx_typecheck_enabled() {
                let corsa_bridge = server.state.get_corsa_bridge().await;
                return Ok(JsxDeclarationService::declaration(&ctx, corsa_bridge).await);
            }
            return Ok(None);
        }

        let corsa_bridge = server.state.get_corsa_bridge().await;
        return Ok(DeclarationService::declaration_with_corsa(&ctx, corsa_bridge).await);
    }

    #[cfg(not(feature = "native"))]
    {
        let _ = params;
        Ok(None)
    }
}

pub(super) async fn goto_implementation(
    server: &MaestroServer,
    params: GotoImplementationParams,
) -> Result<Option<GotoImplementationResponse>> {
    if !server.state.lsp_features().definition || !server.state.lsp_features().typecheck {
        return Ok(None);
    }

    #[cfg(feature = "native")]
    {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(content) = server.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let Some(ctx) = IdeContext::new(&server.state, uri, offset) else {
            return Ok(None);
        };

        if crate::utils::is_jsx_path(uri.path()) {
            if server.state.jsx_typecheck_enabled() {
                let corsa_bridge = server.state.get_corsa_bridge().await;
                return Ok(JsxImplementationService::implementation(&ctx, corsa_bridge).await);
            }
            return Ok(None);
        }

        let corsa_bridge = server.state.get_corsa_bridge().await;
        return Ok(ImplementationService::implementation_with_corsa(&ctx, corsa_bridge).await);
    }

    #[cfg(not(feature = "native"))]
    {
        let _ = params;
        Ok(None)
    }
}
