use super::{DocumentIdentifier, ProjectSession};
use crate::{
    Result,
    lsp_types::{
        CompletionContext, CompletionResponse, GotoDefinitionResponse, Hover, Location,
        WorkspaceEdit,
    },
};

impl ProjectSession {
    /// Returns hover information at a UTF-16 position in the active project.
    pub async fn get_hover_at_position(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<Hover>> {
        self.client()
            .get_hover_at_position(
                self.snapshot().handle.clone(),
                self.project().id.clone(),
                file,
                position,
            )
            .await
    }

    /// Returns definitions at a UTF-16 position in the active project.
    pub async fn get_definition_at_position(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.client()
            .get_definition_at_position(
                self.snapshot().handle.clone(),
                self.project().id.clone(),
                file,
                position,
            )
            .await
    }

    /// Returns references at a UTF-16 position in the active project.
    pub async fn get_references_at_position(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Vec<Location>> {
        self.client()
            .get_references_at_position(
                self.snapshot().handle.clone(),
                self.project().id.clone(),
                file,
                position,
            )
            .await
    }

    /// Returns rename edits at a UTF-16 position in the active project.
    pub async fn get_rename_at_position(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
        new_name: impl Into<String>,
    ) -> Result<Option<WorkspaceEdit>> {
        self.client()
            .get_rename_at_position(
                self.snapshot().handle.clone(),
                self.project().id.clone(),
                file,
                position,
                new_name,
            )
            .await
    }

    /// Returns completions at a UTF-16 position in the active project.
    pub async fn get_completion_at_position(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
        context: Option<CompletionContext>,
    ) -> Result<Option<CompletionResponse>> {
        self.client()
            .get_completion_at_position(
                self.snapshot().handle.clone(),
                self.project().id.clone(),
                file,
                position,
                context,
            )
            .await
    }
}
