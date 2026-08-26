//! Corsa project-session client backed by an adaptive stdio transport.
//!
//! The module path is still `lsp_client` for workspace compatibility, but the
//! implementation now talks directly to `corsa`'s `ProjectSession` APIs.
#![allow(clippy::disallowed_types)]

use corsa::api::{CapabilitiesResponse, ProjectSession};
use lsp_types::Diagnostic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use vize_s0::{FxHashMap, String};

mod bootstrap;
mod diagnostics;
mod diagnostics_api;
mod diagnostics_lsp;
mod editor_lsp;
mod language_id;
mod lifecycle;
mod lifecycle_setup;
mod materialized_refresh;
pub(crate) mod paths;
mod queries;
mod session;
mod session_paths;
mod utils;
mod virtual_overlay;
mod workspace_project;

#[cfg(test)]
mod tests;

/// Thin adapter over `corsa`'s project-session APIs.
pub struct CorsaProjectClient {
    executable: String,
    cwd: PathBuf,
    /// Optional custom project-session transport. Standard tsgo builds expose
    /// the editor LSP without this API, so the client can run editor-only.
    session: Option<ProjectSession>,
    capabilities: Arc<CapabilitiesResponse>,
    overlay_api_disabled: bool,
    materialized_project_session: bool,
    project_root: PathBuf,
    /// Cached diagnostics keyed by document URI.
    pub(crate) diagnostics: FxHashMap<String, Vec<Diagnostic>>,
    /// Per-document overlay versions so Corsa can keep snapshots ordered.
    overlay_versions: FxHashMap<String, i32>,
    /// Current in-memory contents for virtual overlays and offset mapping.
    document_texts: FxHashMap<String, String>,
    /// Mapping from caller-facing URIs to the session-local URIs Corsa sees.
    session_document_uris: FxHashMap<String, String>,
    /// Reverse mapping used to translate API responses back to caller-facing URIs.
    external_document_uris: FxHashMap<String, String>,
    /// Temporary directory for tsconfig.json (cleaned up on drop).
    temp_dir: Option<PathBuf>,
    /// Lazily spawned `--lsp --stdio` session answering editor requests the
    /// project-session API rejects as unsupported (corsa-bind#409), and the
    /// standard-tsgo diagnostics path when no project-session API exists.
    editor_lsp: Option<editor_lsp::EditorLspSession>,
    /// Whether the reusable editor LSP needs the latest virtual project mirror.
    editor_lsp_documents_dirty: bool,
    /// Runtime support for `workspace/willRenameFiles`, probed through the
    /// reusable editor LSP because the project-session API has no equivalent.
    editor_lsp_will_rename_supported: Option<bool>,
    closed: bool,
}

#[cfg(test)]
impl CorsaProjectClient {
    fn empty_for_test(project_root: PathBuf) -> Self {
        Self {
            executable: String::new(""),
            cwd: project_root.clone(),
            session: None,
            capabilities: Arc::new(Default::default()),
            overlay_api_disabled: false,
            materialized_project_session: false,
            project_root,
            diagnostics: Default::default(),
            overlay_versions: Default::default(),
            document_texts: Default::default(),
            session_document_uris: Default::default(),
            external_document_uris: Default::default(),
            temp_dir: None,
            editor_lsp: None,
            editor_lsp_documents_dirty: true,
            editor_lsp_will_rename_supported: None,
            closed: false,
        }
    }
}

/// Legacy name kept for callers that still import `CorsaLspClient`.
pub type CorsaLspClient = CorsaProjectClient;

/// LSP Diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub severity: Option<i32>,
    pub code: Option<Value>,
    pub source: Option<String>,
    pub message: String,
}

/// LSP Range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

/// LSP Position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

pub(crate) struct DiagnosticFetch {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) used_cache: bool,
}

/// Return whether an LSP request failed because its reusable transport became
/// unusable rather than because the request itself was invalid.
///
/// The transport crates currently expose these failures as formatted strings,
/// so this classifier is deliberately narrow. Callers may rebuild a session
/// once for these cases; semantic and configuration errors must propagate.
fn lsp_transport_error_is_transient(error: &str) -> bool {
    error.contains("protocol error: EOF")
        || error.contains("EOF while parsing")
        || error.contains("process is closed: jsonrpc reader")
        || error.contains("Broken pipe")
        || error.contains("broken pipe")
}
