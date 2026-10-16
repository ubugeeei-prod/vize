use super::{
    ApiClient, ApiSpawnConfig, DocumentIdentifier, FileChanges, ManagedSnapshot, NodeHandle,
    OverlayChanges, ProjectHandle, ProjectResponse, SignatureHandle, SignatureResponse,
    SymbolHandle, SymbolResponse, TypeHandle, TypeResponse, UpdateSnapshotParams,
};
use crate::{CorsaError, Result};

/// Snapshot-backed project session for repeated checker queries.
///
/// Corsa exposes most checker APIs in terms of `(snapshot, project, ...)`.
/// This wrapper keeps those two handles alive and refreshes them together so
/// callers can focus on document-level queries instead of transport plumbing.
pub struct ProjectSession {
    client: ApiClient,
    snapshot: ManagedSnapshot,
    project: ProjectResponse,
    open_project: String,
    preferred_document: Option<DocumentIdentifier>,
}

impl ProjectSession {
    /// Spawns a worker and opens a project session in one step.
    pub async fn spawn(
        config: ApiSpawnConfig,
        open_project: impl Into<String>,
        preferred_document: Option<DocumentIdentifier>,
    ) -> Result<Self> {
        let client = ApiClient::spawn(config).await?;
        Self::open(client, open_project, preferred_document).await
    }

    /// Opens a project session from an existing client.
    pub async fn open(
        client: ApiClient,
        open_project: impl Into<String>,
        preferred_document: Option<DocumentIdentifier>,
    ) -> Result<Self> {
        let open_project = open_project.into();
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some(open_project.clone()),
                file_changes: None,
                overlay_changes: None,
            })
            .await?;
        let project = resolve_project(&snapshot, preferred_document.as_ref()).await?;

        Ok(Self {
            client,
            snapshot,
            project,
            open_project,
            preferred_document,
        })
    }

    /// Refreshes the snapshot after file changes and re-resolves the project.
    pub async fn refresh(&mut self, file_changes: Option<FileChanges>) -> Result<()> {
        self.refresh_with_params(UpdateSnapshotParams {
            open_project: None,
            file_changes,
            overlay_changes: None,
        })
        .await
    }

    /// Refreshes the snapshot using explicit update parameters.
    pub async fn refresh_with_params(&mut self, params: UpdateSnapshotParams) -> Result<()> {
        let open_project = params
            .open_project
            .unwrap_or_else(|| self.open_project.clone());
        let snapshot = self
            .client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some(open_project.clone()),
                file_changes: params.file_changes,
                overlay_changes: params.overlay_changes,
            })
            .await?;
        let project = resolve_project(&snapshot, self.preferred_document.as_ref()).await?;
        self.snapshot = snapshot;
        self.project = project;
        self.open_project = open_project;
        Ok(())
    }

    /// Refreshes the snapshot with optional file and overlay changes.
    pub async fn refresh_with_overlay_changes(
        &mut self,
        file_changes: Option<FileChanges>,
        overlay_changes: Option<OverlayChanges>,
    ) -> Result<()> {
        self.refresh_with_params(UpdateSnapshotParams {
            open_project: None,
            file_changes,
            overlay_changes,
        })
        .await?;
        Ok(())
    }

    /// Closes the underlying worker process.
    pub async fn close(&self) -> Result<()> {
        self.client.close().await
    }

    /// Returns the shared low-level client.
    pub fn client(&self) -> &ApiClient {
        &self.client
    }

    /// Returns the current project metadata.
    pub fn project(&self) -> &ProjectResponse {
        &self.project
    }

    /// Returns the active snapshot handle.
    pub fn snapshot(&self) -> &ManagedSnapshot {
        &self.snapshot
    }

    /// Returns the active project handle.
    pub fn project_handle(&self) -> ProjectHandle {
        self.project.id.clone()
    }

    /// Resolves the symbol visible at a UTF-16 position.
    pub async fn get_symbol_at_position(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<SymbolResponse>> {
        self.client
            .get_symbol_at_position(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                file,
                position,
            )
            .await
    }

    /// Resolves the checker type visible at a UTF-16 position.
    pub async fn get_type_at_position(
        &self,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<TypeResponse>> {
        self.client
            .get_type_at_position(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                file,
                position,
            )
            .await
    }

    /// Resolves the apparent checker type of a symbol.
    pub async fn get_type_of_symbol(&self, symbol: SymbolHandle) -> Result<Option<TypeResponse>> {
        self.client
            .get_type_of_symbol(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                symbol,
            )
            .await
    }

    /// Resolves the apparent checker types of multiple symbols.
    pub async fn get_types_of_symbols(
        &self,
        symbols: Vec<SymbolHandle>,
    ) -> Result<Vec<Option<TypeResponse>>> {
        self.client
            .get_types_of_symbols(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                symbols,
            )
            .await
    }

    /// Returns property symbols for a type.
    pub async fn get_properties_of_type(&self, r#type: TypeHandle) -> Result<Vec<SymbolResponse>> {
        self.client
            .get_properties_of_type(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                r#type,
            )
            .await
    }

    /// Returns call signatures for a type.
    pub async fn get_signatures_of_type(
        &self,
        r#type: TypeHandle,
        kind: i32,
    ) -> Result<Vec<SignatureResponse>> {
        self.client
            .get_signatures_of_type(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                r#type,
                kind,
            )
            .await
    }

    /// Returns the return type of a signature.
    pub async fn get_return_type_of_signature(
        &self,
        signature: SignatureHandle,
    ) -> Result<Option<TypeResponse>> {
        self.client
            .get_return_type_of_signature(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                signature,
            )
            .await
    }

    /// Renders a type handle back into TypeScript source text.
    pub async fn type_to_string(
        &self,
        r#type: TypeHandle,
        enclosing: Option<NodeHandle>,
        flags: Option<i32>,
    ) -> Result<String> {
        self.client
            .type_to_string(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                r#type,
                enclosing,
                flags,
            )
            .await
    }
}

async fn resolve_project(
    snapshot: &ManagedSnapshot,
    preferred_document: Option<&DocumentIdentifier>,
) -> Result<ProjectResponse> {
    if let Some(document) = preferred_document {
        if let Some(project) = snapshot
            .get_default_project_for_file(document.clone())
            .await?
        {
            return Ok(project);
        }
    }

    snapshot
        .projects
        .first()
        .cloned()
        .ok_or_else(|| CorsaError::Protocol("project session did not resolve a project".into()))
}
