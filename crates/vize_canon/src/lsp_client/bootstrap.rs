use super::{
    CorsaProjectClient,
    lifecycle_setup::{workspace_config_path, write_materialized_project_tsconfig},
    session::{
        ProjectSessionSpawnError, materialize_session_document, spawn_project_session_with_control,
    },
    session_paths::build_materialized_session_document_uri,
};
use crate::file_uri::file_uri_to_path;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use vize_carton::corsa_resolver::{CorsaResolveError, CorsaResolveRequest};
use vize_l0::{String, cstr};

pub(super) fn resolve_corsa_executable(
    corsa_path: Option<&str>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let request = CorsaResolveRequest {
        explicit_path: corsa_path.map(Path::new),
        project_root: working_dir.map(Path::new),
    };

    match vize_carton::corsa_resolver::resolve_corsa_executable(request) {
        Ok(path) => Ok(path.to_string_lossy().into()),
        // Preserve the historical lenient fallback: spawning a bare `corsa`
        // still lets `PATH` changes made after resolution take effect.
        Err(CorsaResolveError::NotFound) => Ok("corsa".into()),
        Err(error @ CorsaResolveError::ExplicitNotFound { .. }) => Err(cstr!("{error}")),
    }
}

impl CorsaProjectClient {
    pub(super) fn spawn_initialized_client_with_config(
        executable: &str,
        cwd: PathBuf,
        root_path: Option<PathBuf>,
        temp_dir: Option<PathBuf>,
        config_path: Option<&Path>,
    ) -> Result<Self, String> {
        Self::spawn_initialized_client_with_transport(
            executable,
            cwd,
            root_path,
            temp_dir,
            config_path,
            false,
        )
    }

    pub(super) fn spawn_initialized_client_with_transport(
        executable: &str,
        cwd: PathBuf,
        root_path: Option<PathBuf>,
        temp_dir: Option<PathBuf>,
        config_path: Option<&Path>,
        shared_editor_transport: bool,
    ) -> Result<Self, String> {
        Self::spawn_initialized_client_controlled(
            executable,
            cwd,
            root_path,
            temp_dir,
            config_path,
            shared_editor_transport,
            None,
        )
    }

    pub(super) fn spawn_initialized_client_controlled(
        executable: &str,
        cwd: PathBuf,
        root_path: Option<PathBuf>,
        temp_dir: Option<PathBuf>,
        config_path: Option<&Path>,
        shared_editor_transport: bool,
        startup_control: Option<super::startup_control::StartupControl>,
    ) -> Result<Self, String> {
        if let Some(control) = &startup_control {
            control.check_admission()?;
        }
        let project_root = root_path.as_deref().unwrap_or(&cwd);
        let explicit_project_config = config_path.map(Path::to_path_buf);
        let config_path = selected_workspace_config(project_root, config_path);
        let shared = shared_editor_transport
            .then(|| {
                super::editor_lsp::configured_project::try_shared_editor_project(
                    executable,
                    &cwd,
                    project_root,
                    &config_path,
                    startup_control.as_ref(),
                )
            })
            .flatten();
        let (session, capabilities, editor_lsp) =
            if let Some((session, capabilities, editor)) = shared {
                (Some(session), capabilities, Some(editor))
            } else {
                let (session, capabilities) = match spawn_project_session_with_control(
                    executable,
                    &cwd,
                    &config_path,
                    startup_control.as_ref(),
                ) {
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
                (session, capabilities, None)
            };
        let project_session_attached_to_editor = editor_lsp.is_some();
        Ok(Self {
            startup_control,
            executable: executable.into(),
            cwd: cwd.clone(),
            session,
            capabilities,
            overlay_api_disabled: false,
            materialized_project_session: false,
            project_root: project_root.to_path_buf(),
            explicit_project_config,
            diagnostics: Default::default(),
            overlay_versions: Default::default(),
            document_texts: Default::default(),
            session_document_uris: Default::default(),
            external_document_uris: Default::default(),
            temp_dir,
            editor_lsp,
            shared_editor_transport: project_session_attached_to_editor,
            project_session_attached_to_editor,
            original_diagnosing_session: None,
            editor_lsp_documents_dirty: true,
            editor_lsp_will_rename_supported: None,
            closed: false,
        })
    }

    pub(super) fn activate_materialized_project_session(&mut self) -> Result<(), String> {
        if self.materialized_project_session {
            return Ok(());
        }

        self.retire_original_diagnosing_session()?;

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
            self.retire_project_session()?;
            let overlay_root = super::session_paths::overlay_root_for_project(&self.project_root);
            self.replace_project_transport(&config_path, &overlay_root)?;
        }
        self.session_document_uris.clear();
        self.external_document_uris.clear();
        for (uri, document_uri) in mappings {
            self.remember_session_document_uri(uri.as_str(), document_uri);
        }
        self.materialized_project_session = true;
        if self.explicit_project_config.is_some() {
            self.explicit_project_config = Some(config_path);
        }
        // An attached editor was opened against the exact new materialized
        // config above. Independent editor transports still respawn lazily.
        if self.project_session_attached_to_editor {
            Ok(())
        } else {
            self.retire_editor_lsp()
        }
    }
}

fn selected_workspace_config(root: &Path, explicit: Option<&Path>) -> PathBuf {
    explicit
        .map(Path::to_path_buf)
        .unwrap_or_else(|| workspace_config_path(root))
}

#[cfg(test)]
mod tests {
    use super::selected_workspace_config;
    use std::fs;

    #[test]
    fn explicit_generated_config_keeps_nested_authority_with_or_without_root_config() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("playground/tsconfig.json");
        fs::create_dir(nested.parent().unwrap()).unwrap();
        fs::write(&nested, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
        assert_eq!(
            selected_workspace_config(root.path(), Some(&nested)),
            nested
        );
        fs::write(
            root.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":false}}"#,
        )
        .unwrap();
        assert_eq!(
            selected_workspace_config(root.path(), Some(&nested)),
            nested
        );
    }

    #[test]
    fn omitted_config_preserves_root_tsconfig_and_jsconfig_precedence() {
        let root = tempfile::tempdir().unwrap();
        let tsconfig = root.path().join("tsconfig.json");
        let jsconfig = root.path().join("jsconfig.json");
        assert_eq!(selected_workspace_config(root.path(), None), tsconfig);
        fs::write(&jsconfig, "{}").unwrap();
        assert_eq!(selected_workspace_config(root.path(), None), jsconfig);
        fs::write(&tsconfig, "{}").unwrap();
        assert_eq!(selected_workspace_config(root.path(), None), tsconfig);
    }
}
