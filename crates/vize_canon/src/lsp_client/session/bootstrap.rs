//! Project-session transport selection and fallback classification.

use std::{path::Path, sync::Arc};

use corsa::{
    CorsaError,
    api::{ApiMode, ApiSpawnConfig, CapabilitiesResponse, ProjectSession},
    runtime::block_on,
};
use vize_s0::{String, cstr};

pub(in crate::lsp_client) fn spawn_project_session(
    executable: &str,
    cwd: &Path,
    config_path: &Path,
) -> Result<(ProjectSession, Arc<CapabilitiesResponse>), ProjectSessionSpawnError> {
    let config_path_wire = config_path.to_string_lossy();
    let mode = api_mode_for_executable(executable);
    let session = match block_on(spawn_project_session_with_mode(
        executable,
        cwd,
        config_path_wire.as_ref(),
        mode,
    )) {
        Ok(session) => session,
        Err(error) if should_retry_json_rpc(mode, &error) => {
            match block_on(spawn_project_session_with_mode(
                executable,
                cwd,
                config_path_wire.as_ref(),
                ApiMode::AsyncJsonRpcStdio,
            )) {
                Ok(session) => session,
                Err(fallback) => {
                    return Err(classify_project_session_error(
                        fallback,
                        Some(cstr!("after msgpack error: {error}")),
                    ));
                }
            }
        }
        Err(error) => return Err(classify_project_session_error(error, None)),
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
) -> Result<ProjectSession, CorsaError> {
    ProjectSession::spawn(
        ApiSpawnConfig::new(executable)
            .with_mode(mode)
            .with_cwd(cwd),
        config_path,
        None,
    )
    .await
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

pub(super) fn api_mode_for_executable(executable: &str) -> ApiMode {
    if is_node_wrapper_executable(Path::new(executable)) {
        ApiMode::AsyncJsonRpcStdio
    } else {
        ApiMode::SyncMsgpackStdio
    }
}

fn is_node_wrapper_executable(path: &Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) == Some("js") {
        return true;
    }
    if path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        == Some(".bin")
    {
        return true;
    }
    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(grandparent) = parent.parent() else {
        return false;
    };
    parent.file_name().and_then(|name| name.to_str()) == Some("bin")
        && grandparent.file_name().and_then(|name| name.to_str()) == Some("native-preview")
}
