use super::{
    CorsaProjectClient,
    lifecycle_setup::{workspace_config_path, write_materialized_project_tsconfig},
    session::{ProjectSessionSpawnError, materialize_session_document, spawn_project_session},
    session_paths::build_materialized_session_document_uri,
};
use crate::file_uri::file_uri_to_path;
use corsa::runtime::block_on;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use vize_s0::{
    String,
    corsa_resolver::{CorsaResolveError, CorsaResolveRequest},
    cstr,
};

pub(super) fn resolve_corsa_executable(
    corsa_path: Option<&str>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let request = CorsaResolveRequest {
        explicit_path: corsa_path.map(Path::new),
        project_root: working_dir.map(Path::new),
    };

    match vize_s0::corsa_resolver::resolve_corsa_executable(request) {
        Ok(path) => Ok(path.to_string_lossy().into()),
        // Preserve the historical lenient fallback: spawning a bare `corsa`
        // still lets `PATH` changes made after resolution take effect.
        Err(CorsaResolveError::NotFound) => Ok("corsa".into()),
        Err(error @ CorsaResolveError::ExplicitNotFound { .. }) => Err(cstr!("{error}")),
    }
}

impl CorsaProjectClient {
    pub(super) fn spawn_initialized_client(
        executable: &str,
        cwd: PathBuf,
        root_path: Option<PathBuf>,
        temp_dir: Option<PathBuf>,
    ) -> Result<Self, String> {
        let project_root = root_path.as_deref().unwrap_or(&cwd);
        let config_path = workspace_config_path(project_root);
        let (session, capabilities) = match spawn_project_session(executable, &cwd, &config_path) {
            Ok((session, capabilities)) => (Some(session), capabilities),
            Err(ProjectSessionSpawnError::Unavailable(reason)) => {
                tracing::debug!(
                    reason = reason.as_str(),
                    "using standard tsgo editor-only session"
                );
                (None, Arc::new(Default::default()))
            }
            Err(ProjectSessionSpawnError::Failed(error)) => return Err(error),
        };
        Ok(Self {
            executable: executable.into(),
            cwd: cwd.clone(),
            session,
            capabilities,
            overlay_api_disabled: false,
            materialized_project_session: false,
            project_root: project_root.to_path_buf(),
            diagnostics: Default::default(),
            overlay_versions: Default::default(),
            document_texts: Default::default(),
            session_document_uris: Default::default(),
            external_document_uris: Default::default(),
            temp_dir,
            editor_lsp: None,
            editor_lsp_documents_dirty: true,
            editor_lsp_will_rename_supported: None,
            closed: false,
        })
    }

    pub(super) fn activate_materialized_project_session(&mut self) -> Result<(), String> {
        if self.materialized_project_session {
            return Ok(());
        }

        let config_path = write_materialized_project_tsconfig(&self.project_root)?;
        let documents: Vec<_> = self
            .document_texts
            .iter()
            .map(|(uri, content)| (uri.clone(), content.clone()))
            .collect();
        let mut mappings = Vec::with_capacity(documents.len());
        for (uri, content) in &documents {
            let document_uri = build_materialized_session_document_uri(uri, &self.project_root)
                .ok_or_else(|| {
                    cstr!("Failed to derive materialized Corsa overlay path for {uri}")
                })?;
            materialize_session_document(uri, document_uri.as_str(), content)
                .ok_or_else(|| cstr!("Failed to materialize Corsa overlay document for {uri}"))?;
            let path = file_uri_to_path(document_uri.as_str())
                .ok_or_else(|| cstr!("Invalid materialized Corsa document URI {document_uri}"))?;
            let written = std::fs::read_to_string(&path).map_err(|error| {
                cstr!(
                    "Failed to read materialized Corsa document {}: {error}",
                    path.display()
                )
            })?;
            if written != *content {
                return Err(cstr!(
                    "Materialized Corsa document did not preserve contents for {uri}"
                ));
            }
            mappings.push((uri.clone(), document_uri));
        }
        if self.has_project_session() {
            let (session, capabilities) =
                match spawn_project_session(self.executable.as_str(), &self.cwd, &config_path) {
                    Ok(result) => result,
                    Err(ProjectSessionSpawnError::Unavailable(error))
                    | Err(ProjectSessionSpawnError::Failed(error)) => return Err(error),
                };
            let previous = self.session.replace(session);
            if let Some(previous) = previous {
                let _ = block_on(previous.close());
            }
            self.capabilities = capabilities;
        }
        self.session_document_uris.clear();
        self.external_document_uris.clear();
        for (uri, document_uri) in mappings {
            self.remember_session_document_uri(uri.as_str(), document_uri);
        }
        self.materialized_project_session = true;
        // The overlay root moved, so the editor session must be respawned
        // against the materialized tree on the next request.
        self.retire_editor_lsp()
    }
}
