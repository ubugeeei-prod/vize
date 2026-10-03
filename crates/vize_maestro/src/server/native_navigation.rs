//! Explicit preview endpoints; standard LSP routes retain their existing path.
use super::MaestroServer;
use crate::source_project::{SnapshotRefusal, navigation::NavigationRefusal};
use tower_lsp::{
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::{GotoDefinitionParams, Location, ReferenceParams, Url},
};

pub(super) const DEFINITION_METHOD: &str = "vize/nativeDefinition";
pub(super) const REFERENCES_METHOD: &str = "vize/nativeReferences";

impl MaestroServer {
    pub(super) fn notify_native_navigation(&self, uri: &Url) {
        if let Some(project) = &self.navigation {
            project.notify_host_change(uri);
        }
    }

    pub(super) async fn native_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<Location>> {
        let request = params.text_document_position_params;
        self.navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .definition(&request.text_document.uri, request.position)
            .await
            .map_err(query_error)
    }

    pub(super) async fn native_references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        let request = params.text_document_position;
        self.navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .references(
                &request.text_document.uri,
                request.position,
                params.context.include_declaration,
            )
            .await
            .map_err(query_error)
    }
}

fn query_error(refusal: NavigationRefusal) -> Error {
    let (code, message) = match refusal {
        NavigationRefusal::Host(SnapshotRefusal::Cancelled) => {
            (ErrorCode::RequestCancelled, "Native query cancelled")
        }
        NavigationRefusal::Host(SnapshotRefusal::Superseded) => {
            (ErrorCode::ContentModified, "Native source superseded")
        }
        NavigationRefusal::Host(_) => (ErrorCode::InvalidParams, "Native document unavailable"),
        NavigationRefusal::Position => (ErrorCode::InvalidParams, "Invalid native UTF-16 position"),
        NavigationRefusal::Language => (
            ErrorCode::ServerError(-32001),
            "Native navigation language unsupported",
        ),
        NavigationRefusal::Syntax => (
            ErrorCode::ServerError(-32002),
            "Native original Program refused",
        ),
        NavigationRefusal::Producer(_) => (
            ErrorCode::ServerError(-32003),
            "Native File observation refused",
        ),
        NavigationRefusal::Projection => (
            ErrorCode::ServerError(-32004),
            "Native original span projection refused",
        ),
        NavigationRefusal::Query(_) => (
            ErrorCode::ServerError(-32005),
            "Native File position refused",
        ),
        NavigationRefusal::Busy => (
            ErrorCode::ServerError(-32006),
            "Native navigation request queue full",
        ),
        NavigationRefusal::Capacity => (
            ErrorCode::ServerError(-32007),
            "Native navigation worker capacity reached",
        ),
        NavigationRefusal::ConfigurationChanged => (
            ErrorCode::ContentModified,
            "Native Vue configuration superseded",
        ),
        NavigationRefusal::Configuration => (
            ErrorCode::ServerError(-32009),
            "Native Vue configuration unavailable or unsupported",
        ),
        NavigationRefusal::SfcProducer(_) => (
            ErrorCode::ServerError(-32010),
            "Native original SFC observation refused",
        ),
        NavigationRefusal::WorkerUnavailable => (
            ErrorCode::ServerError(-32008),
            "Native navigation worker unavailable",
        ),
    };
    let mut error = Error::new(code);
    error.message = message.into();
    error
}

#[cfg(test)]
mod tests;
