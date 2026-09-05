use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{SignatureHelp, SignatureHelpParams},
};

use super::super::MaestroServer;
#[cfg(feature = "native")]
use crate::ide::{IdeContext, SignatureHelpService, position_to_offset};

pub(super) type SigHelpParams = SignatureHelpParams;
pub(super) type SigHelp = SignatureHelp;

pub(super) async fn signature_help(
    server: &MaestroServer,
    params: SigHelpParams,
) -> Result<Option<SigHelp>> {
    if !server.state.lsp_features().signature_help {
        return Ok(None);
    }

    #[cfg(feature = "native")]
    {
        let context = params
            .context
            .and_then(|context| serde_json::to_value(context).ok());
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(content) = server.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&server.state, uri, offset, content);
        let corsa_bridge = server.state.get_corsa_bridge().await;

        if crate::utils::is_jsx_path(uri.path()) {
            if server.state.jsx_typecheck_enabled() {
                return Ok(crate::ide::JsxService::signature_help_with_context(
                    &ctx,
                    corsa_bridge,
                    context,
                )
                .await);
            }
            return Ok(None);
        }

        return Ok(SignatureHelpService::signature_help_with_corsa_context(
            &ctx,
            corsa_bridge,
            context,
        )
        .await);
    }

    #[cfg(not(feature = "native"))]
    {
        let _ = params;
        Ok(None)
    }
}
