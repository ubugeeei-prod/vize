use super::{
    CorsaProjectClient,
    lifecycle_setup::{workspace_config_path, write_materialized_project_tsconfig},
    session::{ProjectSessionSpawnError, materialize_session_document, spawn_project_session},
    session_paths::build_materialized_session_document_uri,
};
use crate::file_uri::file_uri_to_path;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use vize_carton::corsa_resolver::CorsaResolveRequest;
use vize_l0::{String, cstr};

pub(super) fn resolve_corsa_executable(
    corsa_path: Option<&str>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let request = CorsaResolveRequest {
        explicit_path: corsa_path.map(Path::new),
        project_root: working_dir.map(Path::new),
    };

    vize_carton::corsa_resolver::resolve_corsa_executable(request)
        .map(|path| path.to_string_lossy().into())
        .map_err(|error| cstr!("{error}"))
}

impl CorsaProjectClient {
    pub(super) fn spawn_initialized_client(
        executable: &str,
        cwd: PathBuf,
        root_path: Option<PathBuf>,
        temp_dir: Option<PathBuf>,
    ) -> Result<Self, String> {
        Self::spawn_initialized_client_with_config(executable, cwd, root_path, temp_dir, None)
    }

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
                )
            })
            .flatten();
        let (session, capabilities, editor_lsp) =
            if let Some((session, capabilities, editor)) = shared {
                (Some(session), capabilities, Some(editor))
            } else {
                let (session, capabilities) =
                    match spawn_project_session(executable, &cwd, &config_path) {
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
            shared_editor_transport_requested: shared_editor_transport,
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
    use super::{resolve_corsa_executable, selected_workspace_config};
    use std::fs;

    #[test]
    fn runtime_selection_fixture_preserves_explicit_path_authority() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tests/_fixtures/runtime/native-runtime-10694/selection.json"
        ))
        .unwrap();
        assert_eq!(
            fixture.get("schema").unwrap(),
            "vize.native-runtime-selection"
        );
        assert_eq!(fixture.get("version").unwrap(), 1);
        let mut ids = Vec::new();
        for case in fixture.get("overrideCases").unwrap().as_array().unwrap() {
            if case.get("source").unwrap() != "configuration" {
                continue;
            }
            ids.push(case.get("id").unwrap().as_str().unwrap());
            let root = tempfile::tempdir().unwrap();
            let requested = case.get("requested").unwrap().as_str().unwrap();
            let selected = root.path().join(requested);
            if case.get("present").unwrap().as_bool().unwrap() {
                fs::create_dir_all(selected.parent().unwrap()).unwrap();
                fs::write(&selected, "selection marker; not an executable").unwrap();
            }
            let actual =
                resolve_corsa_executable(Some(requested), Some(root.path().to_str().unwrap()));
            if case.get("present").unwrap().as_bool().unwrap() {
                assert_eq!(
                    std::path::Path::new(actual.unwrap().as_str()),
                    selected.canonicalize().unwrap()
                );
            } else {
                assert_eq!(
                    actual.unwrap_err(),
                    vize_l0::cstr!(
                        "Configured Corsa executable does not exist: {} (from configuration)",
                        selected.display()
                    )
                );
            }
        }
        assert_eq!(ids, ["explicit-valid", "explicit-missing"]);
    }

    #[test]
    fn missing_explicit_runtime_is_an_error_before_any_transport_starts() {
        let root = tempfile::tempdir().unwrap();
        let error = resolve_corsa_executable(
            Some("missing-native-runtime"),
            Some(root.path().to_str().unwrap()),
        )
        .unwrap_err();
        assert!(error.contains("Configured Corsa executable does not exist"));
        assert!(error.contains("missing-native-runtime"));
    }

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
