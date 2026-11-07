//! Exact rename-answer reuse around native request admission.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    PrepareRenameResponse, RenameParams, TextDocumentPositionParams, WorkspaceEdit,
};

use crate::server::project_contexts::DocumentServer;

impl DocumentServer<'_> {
    pub(super) async fn cached_prepare_rename_response(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        #[cfg(feature = "native")]
        let cache_key = if self.state.lsp_features().rename {
            let uri = &params.text_document.uri;
            let position = params.position;
            let stamp = self.state.corsa_request_stamp();
            if let Some(response) = self
                .state
                .cached_native_prepare_rename(uri, position, stamp)
            {
                return self.authored_response(Ok(response));
            }
            Some((uri.clone(), position, stamp))
        } else {
            None
        };
        let response = self
            .native_request(self.prepare_rename_request(params))
            .await?;
        #[cfg(feature = "native")]
        if let Some((uri, position, stamp)) = cache_key {
            self.state
                .cache_native_prepare_rename(&uri, position, stamp, response.clone());
        }
        Ok(response)
    }

    pub(super) async fn cached_rename_response(
        &self,
        params: RenameParams,
    ) -> Result<Option<WorkspaceEdit>> {
        #[cfg(feature = "native")]
        let cache_key = if self.state.lsp_features().rename {
            let request = &params.text_document_position;
            let uri = &request.text_document.uri;
            let position = request.position;
            let stamp = self.state.corsa_request_stamp();
            if let Some(response) =
                self.state
                    .cached_native_rename(uri, position, &params.new_name, stamp)
            {
                return self.authored_response(Ok(response));
            }
            Some((uri.clone(), position, params.new_name.clone(), stamp))
        } else {
            None
        };
        let response = self
            .workspace_native_request(|| self.rename_request(params.clone()))
            .await?;
        #[cfg(feature = "native")]
        if let Some((uri, position, new_name, stamp)) = cache_key {
            self.state
                .cache_native_rename(&uri, position, new_name, stamp, response.clone());
        }
        Ok(response)
    }
}
