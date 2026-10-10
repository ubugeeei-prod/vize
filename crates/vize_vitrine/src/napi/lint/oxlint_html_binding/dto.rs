//! Complete additive DTO for original HTML execution and final presentation.
use napi_derive::napi;
use serde_json::Value;

#[napi(object)]
pub struct OxlintHtmlPresentation {
    /// The actual piped child's graphical theme: plain, unicode, or color.
    pub graphical_theme: String,
    pub links: bool,
    pub width: u32,
    pub stylish_no_color: bool,
    /// Preserve the public wrapper's established cwd-relative Stylish heading.
    pub stylish_relative: Option<bool>,
    /// Original cwd for relative human-readable names; process cwd is untouched.
    pub cwd: String,
}

#[napi(object)]
pub struct OxlintHtmlOptions {
    pub cwd: String,
    pub literal_target: String,
    pub root_json: String,
    /// Exact original caller snapshot, including every whitespace byte.
    #[napi(ts_type = "Array<number>")]
    pub root_bytes: Vec<u8>,
    /// Exact supported host version: 1.78.0 or 1.86.0.
    pub host_profile: String,
    pub no_ignore: bool,
    pub cli_ignore_patterns: Vec<String>,
    pub custom_ignore_filename: String,
    /// Explicit default, json, unix, or stylish.
    pub format: String,
    pub presentation: OxlintHtmlPresentation,
}

#[napi(object)]
pub struct OxlintHtmlOriginal {
    pub path: String,
    pub cwd_relative: String,
    pub origin: String,
    #[napi(ts_type = "Array<number>")]
    pub bytes: Vec<u8>,
}

#[napi(object)]
pub struct OxlintHtmlSource {
    pub path: String,
    pub role: String,
    /// None records absent authority, rather than omitting the authority entry.
    #[napi(ts_type = "Array<number>")]
    pub bytes: Option<Vec<u8>>,
}

#[napi(object)]
pub struct OxlintHtmlRule {
    pub name: String,
    pub severity: Option<String>,
    pub active: bool,
    pub authored_options: Vec<Value>,
}

#[napi(object)]
pub struct OxlintHtmlSettings {
    pub locale: String,
    pub help_level: String,
    pub preset: String,
}

#[napi(object)]
pub struct OxlintHtmlProjection {
    pub rules: Vec<OxlintHtmlRule>,
    pub settings: OxlintHtmlSettings,
    pub deny_warnings: bool,
}

#[napi(object)]
pub struct OxlintHtmlLabel {
    pub message: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct OxlintHtmlDiagnostic {
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub start: u32,
    pub end: u32,
    pub help: Option<String>,
    pub labels: Vec<OxlintHtmlLabel>,
    /// The complete actual fix object, never applied by this operation.
    pub fix: Option<Value>,
}

#[napi(object)]
pub struct OxlintHtmlFile {
    pub path: String,
    pub filename: String,
    pub error_count: f64,
    pub warning_count: f64,
    pub diagnostics: Vec<OxlintHtmlDiagnostic>,
}

#[napi(object)]
pub struct OxlintHtmlCompleted {
    pub host_profile: String,
    pub cwd: String,
    pub literal_target: String,
    pub target: String,
    pub repository: String,
    pub root_json: String,
    pub no_ignore: bool,
    pub cli_ignore_patterns: Vec<String>,
    pub custom_ignore_filename: String,
    pub root_decision: String,
    pub originals: Vec<OxlintHtmlOriginal>,
    pub sources: Vec<OxlintHtmlSource>,
    pub projection: OxlintHtmlProjection,
    pub files: Vec<OxlintHtmlFile>,
    pub executed_file_count: f64,
    pub elapsed_seconds: f64,
    pub errors: f64,
    pub warnings: f64,
    pub output: String,
    /// Actual native diagnostic fragments, without fabricated host report metadata.
    pub json_diagnostics: String,
    pub format: String,
    pub presentation: OxlintHtmlPresentation,
    /// Always not-performed; the CLI must establish genuine host setup authority.
    pub engine_config_validation: String,
}

#[napi(object)]
pub struct OxlintHtmlRefusal {
    pub kind: String,
    pub path: String,
    pub details: String,
    #[napi(ts_type = "Array<number>")]
    pub original_bytes: Option<Vec<u8>>,
}

#[napi(object)]
pub struct OxlintHtmlOutcome {
    /// Exactly one of completed/refused is present.
    pub completed: Option<OxlintHtmlCompleted>,
    pub refused: Option<OxlintHtmlRefusal>,
}
