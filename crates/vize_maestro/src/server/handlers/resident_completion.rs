//! Trigger-scoped completion paths that never acquire a native project.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{CompletionParams, CompletionResponse};

use crate::ide::{CompletionService, IdeContext};
use crate::server::project_contexts::DocumentServer;

impl DocumentServer<'_> {
    pub(super) fn local_resident_component_completion_response(
        &self,
        params: &CompletionParams,
    ) -> Result<Option<Option<CompletionResponse>>> {
        if params
            .context
            .as_ref()
            .and_then(|context| context.trigger_character.as_deref())
            != Some(":")
        {
            return Ok(None);
        }
        self.local_request(|| {
            if !self.state.lsp_features().completion {
                return Ok(None);
            }
            let request = &params.text_document_position;
            let uri = &request.text_document.uri;
            if crate::utils::is_jsx_path(uri.path()) {
                return Ok(None);
            }
            let position = request.position;
            let Some(ctx) = IdeContext::at_completion_position(&self.state, uri, position) else {
                return Ok(None);
            };
            Ok(CompletionService::complete_resident_component_props(&ctx).map(Some))
        })
    }
}
