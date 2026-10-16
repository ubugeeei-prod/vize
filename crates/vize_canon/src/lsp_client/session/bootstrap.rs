//! Project-session transport selection and fallback classification.

use std::{path::Path, sync::Arc};

use corsa::{
    CorsaError,
    api::{ApiClient, ApiMode, ApiSpawnConfig, CapabilitiesResponse, ProjectSession},
    runtime::block_on,
};
use vize_carton::corsa_api_mode::uses_async_json_rpc_api;
use vize_l0::{String, cstr};

pub(in crate::lsp_client) fn spawn_project_session_with_control(
    executable: &str,
    cwd: &Path,
    config_path: &Path,
    control: Option<&crate::lsp_client::startup_control::StartupControl>,
) -> Result<(ProjectSession, Arc<CapabilitiesResponse>), ProjectSessionSpawnError> {
    if let Some(control) = control {
        control
            .check_admission()
            .map_err(ProjectSessionSpawnError::Failed)?;
    }
    let config_path_wire = crate::file_uri::path_to_typescript_name(config_path);
    let mode = api_mode_for_executable(executable);
    let (session, registration) = match spawn_project_session_with_mode(
        executable,
        cwd,
        config_path_wire.as_ref(),
        mode,
        control,
    ) {
        Ok((session, registration)) => (note_project_init(session), registration),
        Err(error) if should_retry_json_rpc(mode, &error) => {
            if let Some(control) = control
                && let Err(closed) = control.check_admission()
            {
                return Err(ProjectSessionSpawnError::Failed(cstr!(
                    "{closed}; refused native fallback after: {error}"
                )));
            }
            match spawn_project_session_with_mode(
                executable,
                cwd,
                config_path_wire.as_ref(),
                ApiMode::AsyncJsonRpcStdio,
                control,
            ) {
                Ok((session, registration)) => (note_project_init(session), registration),
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
            if let Some(control) = control
                && let Err(closed) = control.check_admission()
            {
                return Err(ProjectSessionSpawnError::Failed(cstr!(
                    "{closed}; native startup: {error}"
                )));
            }
            return Err(classify_project_session_error(
                error,
                Some(cstr!("project {config_path_wire}")),
            ));
        }
    };
    let capabilities = match block_on(session.describe_capabilities()) {
        Ok(capabilities) => capabilities,
        Err(error) => {
            crate::lsp_client::retirement::ownership_error(
                &session.client().process_ownership(),
                cstr!("native capabilities: {error}").as_str(),
            );
            if let Some(control) = control
                && let Err(closed) = control.check_admission()
            {
                return Err(ProjectSessionSpawnError::Failed(cstr!(
                    "{closed}; native capabilities: {error}"
                )));
            }
            Arc::new(CapabilitiesResponse::default())
        }
    };
    if let Some(control) = control {
        control
            .check_admission()
            .map_err(ProjectSessionSpawnError::Failed)?;
    }
    drop(registration);
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

fn spawn_project_session_with_mode(
    executable: &str,
    cwd: &Path,
    config_path: &str,
    mode: ApiMode,
    control: Option<&crate::lsp_client::startup_control::StartupControl>,
) -> Result<
    (
        ProjectSession,
        Option<crate::lsp_client::startup_control::Registration>,
    ),
    CorsaError,
> {
    let config = ApiSpawnConfig::new(executable)
        .with_mode(mode)
        .with_cwd(cwd);
    let (client, registration) = match control {
        Some(control) => {
            let (client, registration) = control
                .spawn_api(config)
                .map_err(|error| CorsaError::Protocol(error.as_str().into()))?;
            (client, Some(registration))
        }
        None => (block_on(ApiClient::spawn(config))?, None),
    };
    let owner = client.process_ownership();
    let session = block_on(ProjectSession::open(client, config_path, None)).map_err(|error| {
        crate::lsp_client::retirement::ownership_error(
            &owner,
            cstr!("native project open failed: {error}").as_str(),
        );
        error
    })?;
    Ok((session, registration))
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
