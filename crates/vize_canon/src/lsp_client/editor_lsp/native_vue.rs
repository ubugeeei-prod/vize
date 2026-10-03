//! The actual configured checker owns virtual membership; no tsconfig is rewritten.

use super::{CorsaProjectClient, EditorLspSession};
use crate::corsa_bridge::{CorsaBridgeError, NativeVueError};
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ConfigResponse, ProjectResponse, ProjectSession},
    runtime::block_on,
};
use lsp_types::DocumentDiagnosticReportResult;
use std::{path::Path, sync::Arc};
use vize_l0::cstr;

mod filesystem;
mod project;

impl CorsaProjectClient {
    pub(crate) fn native_vue_diagnostics(
        &mut self,
        uri: &str,
        text: &str,
        config: &Path,
        projected: &Path,
    ) -> Result<
        (
            DocumentDiagnosticReportResult,
            ConfigResponse,
            ProjectResponse,
            std::path::PathBuf,
        ),
        NativeVueError,
    > {
        let backend = |error| NativeVueError::Backend(CorsaBridgeError::CommunicationError(error));
        if self.materialized_project_session || self.document_texts.contains_key(uri) {
            return Err(backend(cstr!(
                "native SFC requires an unmaterialized, unopened projection"
            )));
        }
        let filesystem =
            Arc::new(filesystem::ProjectionFileSystem::new(projected, text).map_err(backend)?);
        let session = block_on(ProjectSession::spawn(
            ApiSpawnConfig::new(self.executable.as_str())
                .with_cwd(&self.cwd)
                .with_mode(
                    if vize_carton::corsa_api_mode::uses_async_json_rpc_api(Path::new(
                        self.executable.as_str(),
                    )) {
                        ApiMode::AsyncJsonRpcStdio
                    } else {
                        ApiMode::SyncMsgpackStdio
                    },
                )
                .with_filesystem(filesystem),
            config
                .to_str()
                .ok_or(NativeVueError::UnconfiguredProjection)?,
            Some(super::super::session::uri_document_identifier(uri)),
        ))
        .map_err(|error| backend(cstr!("cannot open native SFC configured project: {error}")))?;
        let result = (|| {
            let effective = block_on(session.client().parse_config_file(
                super::super::session::uri_document_identifier(&crate::file_uri::path_to_file_uri(
                    config,
                )),
            ))
            .map_err(|error| backend(cstr!("cannot resolve native SFC configuration: {error}")))?;
            let matches = |name: &str| Path::new(name) == projected;
            if !effective.file_names.iter().any(|name| matches(name))
                || !session
                    .project()
                    .root_files
                    .iter()
                    .any(|name| matches(name))
                || Path::new(&session.project().config_file_name) != config
            {
                return Err(NativeVueError::UnconfiguredProjection);
            }
            let project = session.project().clone();
            let mut editor =
                EditorLspSession::spawn(self.executable.as_str(), &self.cwd, &self.project_root)
                    .map_err(backend)?;
            let report = (|| {
                editor.mirror(uri, text).map_err(backend)?;
                let diagnosing_config = editor.native_vue_configuration(uri)?;
                if diagnosing_config != config {
                    return Err(NativeVueError::UnconfiguredProjection);
                }
                let report = editor.diagnostics(uri).map_err(backend)?;
                if editor.native_vue_configuration(uri)? != diagnosing_config {
                    return Err(NativeVueError::UnconfiguredProjection);
                }
                Ok((report, effective, project, diagnosing_config))
            })();
            let cleanup = editor.shutdown().map_err(backend);
            match (report, cleanup) {
                (Ok(report), Ok(())) => Ok(report),
                (Err(error), _) | (_, Err(error)) => Err(error),
            }
        })();
        let cleanup = block_on(session.close())
            .map_err(|error| backend(cstr!("cannot close native SFC configured project: {error}")));
        match (result, cleanup) {
            (Ok(report), Ok(())) => Ok(report),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }
}
