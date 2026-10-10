//! Explicit native endpoints and the opt-in standard linked-editing route.
use super::MaestroServer;
use crate::source_project::{SnapshotRefusal, navigation::NavigationRefusal};
use tower_lsp::{
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::{
        DocumentHighlight, DocumentHighlightParams, GotoDefinitionParams, LinkedEditingRangeParams,
        LinkedEditingRanges, Location, ReferenceParams, Url,
    },
};

pub(super) const DEFINITION_METHOD: &str = "vize/nativeDefinition";
pub(super) const REFERENCES_METHOD: &str = "vize/nativeReferences";
pub(super) const TEMPLATE_DEFINITION_METHOD: &str = "vize/nativeTemplateDefinition";
pub(super) const TEMPLATE_REFERENCES_METHOD: &str = "vize/nativeTemplateReferences";
pub(super) const HIGHLIGHTS_METHOD: &str = "vize/nativeDocumentHighlight";
pub(super) const TEMPLATE_HIGHLIGHTS_METHOD: &str = "vize/nativeTemplateDocumentHighlight";
pub(super) const MODULE_LINKS_METHOD: &str = "vize/nativeModuleDocumentLinks";
mod module_links;

impl MaestroServer {
    pub(super) async fn native_linked_editing(
        &self,
        params: LinkedEditingRangeParams,
        ticket: super::NativeLinkedNamesTicket,
    ) -> Result<Option<LinkedEditingRanges>> {
        let request = params.text_document_position_params;
        self.navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .linked_editing_configured(&request.text_document.uri, request.position, ticket)
            .await
            .map_err(query_error)
    }

    pub(super) async fn native_highlights(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Vec<DocumentHighlight>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        let request = params.text_document_position_params;
        server
            .navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .highlights(&request.text_document.uri, request.position)
            .await
            .map_err(query_error)
    }

    pub(super) async fn native_template_highlights(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Vec<DocumentHighlight>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        let request = params.text_document_position_params;
        server
            .navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .template_highlights(&request.text_document.uri, request.position)
            .await
            .map_err(query_error)
    }
    pub(super) async fn native_template_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Vec<Location>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        let request = params.text_document_position_params;
        server
            .navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .template_definition(&request.text_document.uri, request.position)
            .await
            .map_err(query_error)
    }

    pub(super) async fn native_template_references(
        &self,
        params: ReferenceParams,
    ) -> Result<Vec<Location>> {
        let server = self.for_pos(&params.text_document_position).await;
        let request = params.text_document_position;
        server
            .navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .template_references(
                &request.text_document.uri,
                request.position,
                params.context.include_declaration,
            )
            .await
            .map_err(query_error)
    }

    pub(super) fn notify_native_navigation(&self, uri: &Url) {
        if let Some(project) = &self.navigation {
            project.notify_host_change(uri);
        }
    }

    pub(super) async fn native_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<Location>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        let request = params.text_document_position_params;
        server
            .navigation
            .as_ref()
            .ok_or_else(Error::internal_error)?
            .definition(&request.text_document.uri, request.position)
            .await
            .map_err(query_error)
    }

    pub(super) async fn native_references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        let server = self.for_pos(&params.text_document_position).await;
        let request = params.text_document_position;
        server
            .navigation
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
        NavigationRefusal::Query(_)
        | NavigationRefusal::NativeQuery(_)
        | NavigationRefusal::TemplateQuery(_) => (
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
        NavigationRefusal::NamesRouteChanged => (ErrorCode::ContentModified, "Content modified"),
        NavigationRefusal::Configuration => (
            ErrorCode::ServerError(-32009),
            "Native Vue configuration unavailable or unsupported",
        ),
        NavigationRefusal::SfcProducer(_) => (
            ErrorCode::ServerError(-32010),
            "Native original SFC observation refused",
        ),
        NavigationRefusal::SelectedSfcProducer(_) => (
            ErrorCode::ServerError(-32011),
            "Native original selected SFC observation refused",
        ),
        NavigationRefusal::TemplateNamesProducer
        | NavigationRefusal::TemplateFrameNames(_)
        | NavigationRefusal::ElementNames(_) => (
            ErrorCode::ServerError(-32012),
            "Native original template names refused",
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
