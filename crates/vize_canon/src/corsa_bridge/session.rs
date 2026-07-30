//! Corsa project-session construction, run on the bridge worker thread.
#![allow(clippy::disallowed_types)]

use super::types::{CorsaBridgeConfig, CorsaBridgeError};
use crate::corsa_client::CorsaProjectClient;

/// Spawn the Corsa project session described by `config`.
///
/// Runs on the bridge worker thread (see [`super::worker`]): the handshake
/// this performs is synchronous IPC and is exactly as prone to hanging as any
/// later request, so it has to be bounded the same way.
pub(super) fn build_client(
    config: &CorsaBridgeConfig,
) -> Result<CorsaProjectClient, CorsaBridgeError> {
    let corsa_path = config
        .corsa_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());
    let working_dir = config
        .working_dir
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());

    // Root the session at the real workspace when it is a TypeScript
    // project: its own tsconfig (paths, baseUrl) then drives module
    // resolution and virtual `.vue.ts` overlays can live at their real
    // paths, so relative imports in `<script>` resolve exactly like
    // `vize check`. The isolated scratch session — which synthesizes a
    // tsconfig and `*.vue` stubs — remains the fallback for rootless or
    // tsconfig-less usage.
    let workspace_root = working_dir
        .as_deref()
        .map(std::path::Path::new)
        .filter(|dir| dir.join("tsconfig.json").is_file() || dir.join("jsconfig.json").is_file());

    match workspace_root {
        Some(dir) => CorsaProjectClient::new_for_workspace(corsa_path.as_deref(), dir),
        None => CorsaProjectClient::new(corsa_path.as_deref(), working_dir.as_deref()),
    }
    .map_err(CorsaBridgeError::SpawnFailed)
}
