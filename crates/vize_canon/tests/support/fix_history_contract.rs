//! Existing fixture schemas; visibility changes only permit the moved consumers.

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub(crate) file: String,
    pub(crate) source: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Diagnostic {
    pub(crate) file: String,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) severity: u8,
    pub(crate) code: Option<u32>,
    pub(crate) message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Case {
    pub(crate) id: String,
    pub(crate) inputs: Vec<Input>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Contract {
    #[serde(rename = "requiredTier")]
    pub(crate) required_tier: String,
    #[serde(rename = "positionBase")]
    pub(crate) position_base: u32,
    pub(crate) positions: String,
    pub(crate) comparison: String,
    #[serde(rename = "missingFields")]
    pub(crate) missing_fields: Vec<String>,
    pub(crate) native: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CheckerOptions {
    pub(crate) options_api: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Pack {
    pub(crate) version: u32,
    pub(crate) issue: u32,
    pub(crate) regression_commit: String,
    pub(crate) historical_issue: Option<u32>,
    pub(crate) source_revision: String,
    pub(crate) diagnostic_contract: Contract,
    pub(crate) checker_options: Option<CheckerOptions>,
    pub(crate) project_options: Option<serde_json::Value>,
    pub(crate) cases: Vec<Case>,
}
