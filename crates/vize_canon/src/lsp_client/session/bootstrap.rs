//! Project-session transport selection and fallback classification.

use std::{path::Path, sync::Arc};

use corsa::{
    CorsaError,
    api::{ApiMode, ApiSpawnConfig, CapabilitiesResponse, ProjectSession},
    runtime::block_on,
};
use vize_carton::corsa_api_mode::uses_async_json_rpc_api;
use vize_l0::{String, cstr};

pub(in crate::lsp_client) fn spawn_project_session(
    executable: &str,
    cwd: &Path,
    config_path: &Path,
) -> Result<(ProjectSession, Arc<CapabilitiesResponse>), ProjectSessionSpawnError> {
    spawn_project_session_for_document(executable, cwd, config_path, None)
}

pub(in crate::lsp_client) fn spawn_project_session_for_document(
    executable: &str,
    cwd: &Path,
    config_path: &Path,
    preferred: Option<&str>,
) -> Result<(ProjectSession, Arc<CapabilitiesResponse>), ProjectSessionSpawnError> {
    let config_path_wire = crate::file_uri::path_to_typescript_name(config_path);
    let mode = api_mode_for_executable(executable);
    let session = match block_on(spawn_project_session_with_mode(
        executable,
        cwd,
        config_path_wire.as_ref(),
        mode,
        preferred,
    )) {
        Ok(session) => note_project_init(session),
        Err(error) if should_retry_json_rpc(mode, &error) => {
            match block_on(spawn_project_session_with_mode(
                executable,
                cwd,
                config_path_wire.as_ref(),
                ApiMode::AsyncJsonRpcStdio,
                preferred,
            )) {
                Ok(session) => note_project_init(session),
                Err(fallback) => {
                    return Err(classify_project_session_error(
                        fallback,
                        Some(cstr!(
                            "after msgpack error: {error}; project {config_path_wire}"
                        )),
                    ));
                }
            }
        }
        Err(error) => {
            return Err(classify_project_session_error(
                error,
                Some(cstr!("project {config_path_wire}")),
            ));
        }
    };
    let capabilities = block_on(session.describe_capabilities())
        .unwrap_or_else(|_| Arc::new(CapabilitiesResponse::default()));
    Ok((session, capabilities))
}

#[derive(Debug)]
pub(in crate::lsp_client) enum ProjectSessionSpawnError {
    Unavailable(String),
    Failed(String),
}

pub(super) fn classify_project_session_error(
    error: CorsaError,
    context: Option<String>,
) -> ProjectSessionSpawnError {
    let message = context.map_or_else(
        || cstr!("Failed to start Corsa API session: {error}"),
        |context| cstr!("Failed to start Corsa API session: {error} ({context})"),
    );
    if matches!(
        &error,
        CorsaError::Protocol(detail)
            if detail.contains("project session did not resolve a project")
    ) {
        ProjectSessionSpawnError::Unavailable(message)
    } else {
        ProjectSessionSpawnError::Failed(message)
    }
}

async fn spawn_project_session_with_mode(
    executable: &str,
    cwd: &Path,
    config_path: &str,
    mode: ApiMode,
    preferred: Option<&str>,
) -> Result<ProjectSession, CorsaError> {
    ProjectSession::spawn(
        crate::corsa_profile::with_request_profiling(
            ApiSpawnConfig::new(executable)
                .with_mode(mode)
                .with_cwd(cwd),
            crate::corsa_profile::RequestOrigin::Canon,
        ),
        config_path,
        preferred.map(super::uri_document_identifier),
    )
    .await
}

pub(in crate::lsp_client) fn validate_materialized_project_selection(
    project: &corsa::api::ProjectResponse,
    config: &Path,
    preferred: Option<&str>,
) -> Result<(), String> {
    let Some(preferred) = preferred else {
        return Ok(());
    };
    let expected_config = config
        .canonicalize()
        .map_err(|error| cstr!("Cannot inspect materialized Corsa config: {error}"))?;
    let selected = Path::new(project.config_file_name.as_str())
        .canonicalize()
        .map_err(|error| cstr!("Cannot inspect selected materialized Corsa config: {error}"))?;
    let member = crate::file_uri::file_uri_to_path(preferred)
        .ok_or_else(|| cstr!("Invalid preferred materialized Corsa document"))?
        .canonicalize()
        .map_err(|error| cstr!("Cannot inspect preferred materialized Corsa member: {error}"))?;
    if selected != expected_config
        || !project.root_files.iter().any(|file| {
            Path::new(file.as_str())
                .canonicalize()
                .is_ok_and(|path| path == member)
        })
    {
        return Err(cstr!(
            "Preferred materialized Corsa member did not retain its exact configured project"
        ));
    }
    Ok(())
}

pub(super) fn should_retry_json_rpc(mode: ApiMode, error: &CorsaError) -> bool {
    if mode != ApiMode::SyncMsgpackStdio {
        return false;
    }
    let CorsaError::Protocol(message) = error else {
        return false;
    };
    let message = message.as_str();
    message.contains("expected tuple marker")
        || message.contains("expected uint8 marker")
        || message.contains("expected bin marker")
}

fn note_project_init(session: ProjectSession) -> ProjectSession {
    crate::corsa_session_cache::note_typescript_project_init();
    session
}

pub(in crate::lsp_client) fn api_mode_for_executable(executable: &str) -> ApiMode {
    if uses_async_json_rpc_api(Path::new(executable)) {
        ApiMode::AsyncJsonRpcStdio
    } else {
        ApiMode::SyncMsgpackStdio
    }
}
