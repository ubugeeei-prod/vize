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
use vize_carton::{FxHashMap, String};

mod bootstrap;
mod diagnostics;
mod diagnostics_api;
mod lifecycle;
pub(crate) mod paths;
mod queries;
mod session;
mod utils;

#[cfg(test)]
mod tests;

/// Thin adapter over `corsa`'s project-session APIs.
pub struct CorsaProjectClient {
    executable: String,
    cwd: PathBuf,
    session: ProjectSession,
    capabilities: Arc<CapabilitiesResponse>,
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
    closed: bool,
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
