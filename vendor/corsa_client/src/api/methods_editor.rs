//! Editor-style `ApiClient` methods.

use super::{
    ApiClient, DocumentIdentifier, ProjectHandle, SnapshotHandle,
    requests_core::SymbolAtPositionRequest,
    requests_editor::{CompletionAtPositionRequest, RenameAtPositionRequest},
};
use crate::{
    Result,
    lsp_types::{
        CompletionContext, CompletionResponse, GotoDefinitionResponse, Hover, Location,
        WorkspaceEdit,
    },
};

impl ApiClient {
    /// Returns hover information at a UTF-16 position.
    pub async fn get_hover_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<Hover>> {
        self.call_optional("getHoverAtPosition", SymbolAtPositionRequest {
            snapshot,
            project,
            file: file.into(),
            position,
        })
        .await
        .map_err(|error| {
            ApiClient::map_missing_method(
                error,
                "hover is not supported by this runtime; check describeCapabilities before requesting editor features",
            )
        })
    }

    /// Returns definitions reachable from a UTF-16 position.
    pub async fn get_definition_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.call_optional("getDefinitionAtPosition", SymbolAtPositionRequest {
            snapshot,
            project,
            file: file.into(),
            position,
        })
        .await
        .map_err(|error| {
            ApiClient::map_missing_method(
                error,
                "definition lookup is not supported by this runtime; check describeCapabilities before requesting editor features",
            )
        })
    }

    /// Returns references reachable from a UTF-16 position.
    pub async fn get_references_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Vec<Location>> {
        self.call::<Option<Vec<Location>>, _>(
            "getReferencesAtPosition",
            SymbolAtPositionRequest {
                snapshot,
                project,
                file: file.into(),
                position,
            },
        )
        .await
        .map(|locations| locations.unwrap_or_default())
        .map_err(|error| {
            ApiClient::map_missing_method(
                error,
                "reference lookup is not supported by this runtime; check describeCapabilities before requesting editor features",
            )
        })
    }

    /// Returns rename edits reachable from a UTF-16 position.
    pub async fn get_rename_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
        new_name: impl Into<String>,
    ) -> Result<Option<WorkspaceEdit>> {
        self.call_optional(
            "getRenameAtPosition",
            RenameAtPositionRequest {
                snapshot,
                project,
                file: file.into(),
                position,
                new_name: new_name.into(),
            },
        )
        .await
        .map_err(|error| {
            ApiClient::map_missing_method(
                error,
                "rename is not supported by this runtime; check describeCapabilities before requesting editor features",
            )
        })
    }

    /// Returns completions reachable from a UTF-16 position.
    pub async fn get_completion_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
        context: Option<CompletionContext>,
    ) -> Result<Option<CompletionResponse>> {
        self.call_optional(
            "getCompletionAtPosition",
            CompletionAtPositionRequest {
                snapshot,
                project,
                file: file.into(),
                position,
                context,
            },
        )
        .await
        .map_err(|error| {
            ApiClient::map_missing_method(
                error,
                "completion is not supported by this runtime; check describeCapabilities before requesting editor features",
            )
        })
    }
}
