//! Native checks attest normalized options through the diagnosing process's API.

use super::EditorLspSession;
use crate::corsa_bridge::DiagnosingSnapshot;
use corsa::{
    api::{ApiClient, UpdateSnapshotParams},
    runtime::block_on,
};
#[cfg(unix)]
use corsa_lsp::InitializeApiSessionRequest;
use corsa_lsp::InitializeApiSessionResult;
use serde_json::Value;
use std::path::Path;
use vize_l0::{String, cstr};

pub(super) enum ConfigurationError {
    Communication(String),
    Unconfigured,
    Changed,
    #[cfg(not(unix))]
    Unsupported,
}

pub(super) struct DiagnosingApi {
    pub(super) client: ApiClient,
    pub(super) session: InitializeApiSessionResult,
}

impl EditorLspSession {
    #[cfg(unix)]
    pub(super) fn diagnosing_api(
        &mut self,
        uri: &str,
    ) -> Result<DiagnosingApi, ConfigurationError> {
        self.ready_document_uri(uri)
            .map_err(ConfigurationError::Communication)?;
        self.attach_api()
    }

    #[cfg(unix)]
    pub(super) fn attach_api(&self) -> Result<DiagnosingApi, ConfigurationError> {
        let session = block_on(
            self.client
                .request::<InitializeApiSessionRequest>(Default::default()),
        )
        .map_err(|error| {
            ConfigurationError::Communication(cstr!("cannot attach diagnosing API: {error}"))
        })?;
        let client = block_on(ApiClient::connect_pipe(session.pipe.as_str())).map_err(|error| {
            ConfigurationError::Communication(cstr!("cannot connect diagnosing API: {error}"))
        })?;
        Ok(DiagnosingApi { client, session })
    }

    #[cfg(not(unix))]
    pub(super) fn diagnosing_api(
        &mut self,
        _uri: &str,
    ) -> Result<DiagnosingApi, ConfigurationError> {
        // The pinned SDK exposes no real named-pipe attachment on this target.
        Err(ConfigurationError::Unsupported)
    }

    #[cfg(not(unix))]
    pub(super) fn attach_api(&self) -> Result<DiagnosingApi, ConfigurationError> {
        Err(ConfigurationError::Unsupported)
    }
}

impl DiagnosingApi {
    pub(super) fn observe(
        &self,
        uri: &str,
        config: &Path,
        admitted_options: &Value,
    ) -> Result<DiagnosingSnapshot, ConfigurationError> {
        self.observe_with_params(uri, config, admitted_options, Default::default())
    }

    pub(super) fn observe_with_params(
        &self,
        uri: &str,
        config: &Path,
        admitted_options: &Value,
        params: UpdateSnapshotParams,
    ) -> Result<DiagnosingSnapshot, ConfigurationError> {
        let communication = |error| {
            ConfigurationError::Communication(cstr!("cannot observe diagnosing options: {error}"))
        };
        let snapshot = block_on(self.client.update_snapshot(params)).map_err(communication)?;
        let result = (|| {
            let project =
                block_on(snapshot.get_default_project_for_file(
                    super::super::session::uri_document_identifier(uri),
                ))
                .map_err(communication)?
                .ok_or(ConfigurationError::Unconfigured)?;
            if Path::new(&project.config_file_name) != config {
                return Err(ConfigurationError::Unconfigured);
            }
            // Both endpoints serialize the actual core.CompilerOptions value.
            // Compare the complete value, including inherited paths/options.
            if &project.compiler_options != admitted_options {
                return Err(ConfigurationError::Changed);
            }
            Ok(DiagnosingSnapshot {
                handle: snapshot.handle.clone(),
                projects: snapshot.projects.clone(),
                changes: snapshot.changes.clone(),
                project,
            })
        })();
        let cleanup = block_on(snapshot.release()).map_err(communication);
        match (result, cleanup) {
            (Ok(observed), Ok(())) => Ok(observed),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    pub(super) fn close(&self, editor: &mut EditorLspSession) -> Result<(), ConfigurationError> {
        // The attached SDK reader keeps a full-duplex socket clone. Reap the
        // owning LSP first so the peer closes, then drain and join that reader.
        // observe() has already released each snapshot while the backend lived.
        let owner = editor.shutdown().map_err(ConfigurationError::Communication);
        let attachment = self.close_attachment();
        match (owner, attachment) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    pub(super) fn close_attachment(&self) -> Result<(), ConfigurationError> {
        block_on(self.client.close()).map_err(|error| {
            ConfigurationError::Communication(cstr!("cannot close diagnosing API: {error}"))
        })
    }
}

#[cfg(all(test, unix))]
mod tests {
    //! Independent real backend evidence for the stale-admission failure.

    use super::*;
    use corsa::api::{ApiMode, ApiSpawnConfig};

    #[test]
    fn actual_inherited_options_mismatch_retains_the_complete_new_backend_diagnostic() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let executable = vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
                project_root: Some(repo),
            },
        )
        .unwrap();
        let root = tempfile::TempDir::new().unwrap();
        let config = root.path().join("tsconfig.json");
        let base = root.path().join("base.json");
        let configuration = r#"{"extends":"./base.json","include":["source.ts"]}"#;
        let initial = r#"{"compilerOptions":{"strict":false,"moduleDetection":"force","types":[],"noEmit":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext"}}"#;
        std::fs::write(&config, configuration).unwrap();
        std::fs::write(&base, initial).unwrap();
        let path = root.path().join("source.ts");
        let source = "const value=null; value.toFixed();";
        std::fs::write(&path, source).unwrap();
        let admission = block_on(ApiClient::spawn(
            ApiSpawnConfig::new(&executable)
                .with_cwd(root.path())
                .with_mode(ApiMode::AsyncJsonRpcStdio),
        ))
        .unwrap();
        let effective = block_on(admission.parse_config_file(
            super::super::super::session::uri_document_identifier(
                &crate::file_uri::path_to_file_uri(&config),
            ),
        ))
        .unwrap();
        assert_eq!(effective.options["strict"], false);
        block_on(admission.close()).unwrap();
        std::fs::write(
            &base,
            initial.replace("\"strict\":false", "\"strict\":true"),
        )
        .unwrap();
        let uri = crate::file_uri::path_to_file_uri(&path);
        let mut editor =
            EditorLspSession::spawn(executable.to_str().unwrap(), root.path(), root.path())
                .unwrap();
        editor.mirror(&uri, source).unwrap();
        let api = editor
            .diagnosing_api(&uri)
            .unwrap_or_else(|_| panic!("real attachment required"));
        let report = editor.diagnostics(&uri).unwrap();
        let raw = serde_json::to_value(report).unwrap();
        assert_eq!(
            raw["items"],
            serde_json::json!([{
                "range":{"start":{"line":0,"character":18},"end":{"line":0,"character":23}},
                "severity":1,"code":18047,"source":"ts","message":"'value' is possibly 'null'."
            }])
        );
        assert!(matches!(
            api.observe(&uri, &config, &effective.options),
            Err(ConfigurationError::Changed)
        ));
        api.close(&mut editor)
            .unwrap_or_else(|_| panic!("real owner/attachment cleanup required"));
        assert_eq!(std::fs::read(&config).unwrap(), configuration.as_bytes());
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    }
}
