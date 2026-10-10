//! Explicit point links, with unchanged standard DocumentLink dispatch.
use super::{MaestroServer, query_error};
use crate::source_project::navigation::ModuleDocumentLinkError;
use tower_lsp::{
    jsonrpc::{Error, Result},
    lsp_types::{DocumentLink, DocumentLinkParams},
};

impl MaestroServer {
    pub(in crate::server) async fn native_module_document_links(
        &self,
        params: DocumentLinkParams,
    ) -> Result<Vec<DocumentLink>> {
        let server = self.for_document(&params.text_document.uri).await;
        server
            .navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .module_document_links(&params.text_document.uri)
            .await
            .map_err(module_link_error)
    }
}

fn module_link_error(error: ModuleDocumentLinkError) -> Error {
    error.into_rpc_error(query_error)
}
