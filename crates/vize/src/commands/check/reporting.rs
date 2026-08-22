//! Reporting data structures for `vize check`.

use serde::Serialize;

/// JSON output structure for `--format json`.
#[derive(Serialize)]
#[allow(clippy::disallowed_types)]
pub(crate) struct JsonOutput {
    pub files: Vec<JsonFileResult>,
    pub programs: Vec<JsonProgramResult>,
    #[serde(rename = "errorCount")]
    pub error_count: usize,
    #[serde(rename = "warningCount")]
    pub warning_count: usize,
    #[serde(rename = "fileCount")]
    pub file_count: usize,
    #[serde(rename = "declarations", skip_serializing_if = "Option::is_none")]
    pub declarations: Option<Vec<std::string::String>>,
}

/// Effective TypeScript program evidence in JSON output.
#[derive(Serialize)]
#[allow(clippy::disallowed_types)]
pub(crate) struct JsonProgramResult {
    pub root: std::string::String,
    #[serde(rename = "tsconfig", skip_serializing_if = "Option::is_none")]
    pub tsconfig: Option<std::string::String>,
    pub files: Vec<std::string::String>,
}

/// Per-file result in JSON output.
#[derive(Serialize)]
#[allow(clippy::disallowed_types)]
pub(crate) struct JsonFileResult {
    pub file: std::string::String,
    #[serde(rename = "virtualTs", skip_serializing_if = "Option::is_none")]
    pub virtual_ts: Option<std::string::String>,
    pub diagnostics: Vec<std::string::String>,
}
