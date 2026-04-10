//! Reporting data structures for `vize check`.

use serde::Serialize;

/// JSON output structure for `--format json`.
#[derive(Serialize)]
#[allow(clippy::disallowed_types)]
pub(crate) struct JsonOutput {
    pub files: Vec<JsonFileResult>,
    #[serde(rename = "errorCount")]
    pub error_count: usize,
    #[serde(rename = "fileCount")]
    pub file_count: usize,
    #[serde(rename = "declarations", skip_serializing_if = "Option::is_none")]
    pub declarations: Option<Vec<std::string::String>>,
}

/// Per-file result in JSON output.
#[derive(Serialize)]
#[allow(clippy::disallowed_types)]
pub(crate) struct JsonFileResult {
    pub file: std::string::String,
    #[serde(rename = "virtualTs")]
    pub virtual_ts: std::string::String,
    pub diagnostics: Vec<std::string::String>,
}
