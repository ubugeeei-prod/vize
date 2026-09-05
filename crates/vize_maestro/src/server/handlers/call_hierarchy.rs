use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CallHierarchyIncomingCall, CallHierarchyIncomingCallsParams, CallHierarchyItem,
        CallHierarchyOutgoingCall, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
    },
};

use super::super::MaestroServer;
#[cfg(feature = "native")]
use crate::ide::{CallHierarchyService, IdeContext, position_to_offset};

pub(super) type CHPrepareParams = CallHierarchyPrepareParams;
pub(super) type CHItems = Vec<CallHierarchyItem>;
pub(super) type CHIncomingParams = CallHierarchyIncomingCallsParams;
pub(super) type CHIncomingResponse = Vec<CallHierarchyIncomingCall>;
pub(super) type CHOutgoingParams = CallHierarchyOutgoingCallsParams;
pub(super) type CHOutgoingResponse = Vec<CallHierarchyOutgoingCall>;

pub(super) async fn prepare(
    server: &MaestroServer,
    params: CHPrepareParams,
) -> Result<Option<CHItems>> {
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

        let ctx = IdeContext::with_content(&server.state, uri, offset, content);
        if crate::utils::is_jsx_path(uri.path()) {
            return Ok(None);
        }

        let corsa_bridge = server.state.get_corsa_bridge().await;
        return Ok(CallHierarchyService::prepare_with_corsa(&ctx, corsa_bridge).await);
    }

    #[cfg(not(feature = "native"))]
    {
        let _ = params;
        Ok(None)
    }
}
