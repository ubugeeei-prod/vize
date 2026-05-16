//! Type definitions for the Corsa bridge.
//!
//! Contains all LSP protocol types, error types, configuration,
//! and result types used by the bridge.
//!
//! Many structs use `std::string::String` for serde deserialization compatibility.
#![allow(clippy::disallowed_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

use vize_carton::source_range::SourceMap;
use vize_carton::String;

/// Virtual URI scheme for in-memory documents.
pub const VIRTUAL_URI_SCHEME: &str = "vize-virtual";

/// Error types for Corsa bridge operations.
#[derive(Debug, Clone)]
pub enum CorsaBridgeError {
    /// Failed to spawn the Corsa process.
    SpawnFailed(String),
    /// Failed to communicate with Corsa.
    CommunicationError(String),
    /// Corsa returned an error response.
    ResponseError { code: i64, message: String },
    /// Request timed out
    Timeout,
    /// Bridge is not initialized
    NotInitialized,
    /// Process has terminated
    ProcessTerminated,
}

impl std::fmt::Display for CorsaBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpawnFailed(msg) => write!(f, "Failed to spawn Corsa: {}", msg),
            Self::CommunicationError(msg) => write!(f, "Communication error: {}", msg),
            Self::ResponseError { code, message } => {
                write!(f, "corsa error [{}]: {}", code, message)
            }
            Self::Timeout => write!(f, "Request timed out"),
            Self::NotInitialized => write!(f, "Bridge not initialized"),
            Self::ProcessTerminated => write!(f, "Corsa process terminated"),
        }
    }
}

impl std::error::Error for CorsaBridgeError {}

/// LSP diagnostic from Corsa.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::disallowed_types)]
pub struct LspDiagnostic {
    /// Diagnostic range
    pub range: LspRange,
    /// Severity (1=Error, 2=Warning, 3=Info, 4=Hint)
    pub severity: Option<u8>,
    /// Diagnostic code
    pub code: Option<Value>,
    /// Source (e.g., "ts")
    pub source: Option<std::string::String>,
    /// Message
    pub message: std::string::String,
    /// Related information
    #[serde(rename = "relatedInformation")]
    pub related_information: Option<Vec<LspRelatedInformation>>,
}

/// LSP range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

/// LSP position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

/// Related diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::disallowed_types)]
pub struct LspRelatedInformation {
    pub location: LspLocation,
    pub message: std::string::String,
}

/// LSP location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::disallowed_types)]
pub struct LspLocation {
    pub uri: std::string::String,
    pub range: LspRange,
}

/// LSP hover response.
#[derive(Debug, Clone, Deserialize)]
pub struct LspHover {
    /// The hover's content
    pub contents: LspHoverContents,
    /// An optional range
    pub range: Option<LspRange>,
}

/// LSP hover contents - can be markup or multiple items.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(clippy::disallowed_types)]
pub enum LspHoverContents {
    /// A single MarkupContent
    Markup(LspMarkupContent),
    /// A single string
    String(std::string::String),
    /// Array of marked strings or MarkupContent
    Array(Vec<LspMarkedString>),
}

/// LSP markup content.
#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::disallowed_types)]
pub struct LspMarkupContent {
    /// The type of the Markup ("markdown" | "plaintext")
    pub kind: std::string::String,
    /// The content itself
    pub value: std::string::String,
}

/// LSP marked string (for hover arrays).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(clippy::disallowed_types)]
pub enum LspMarkedString {
    /// A simple string
    String(std::string::String),
    /// Language-tagged code block
    LanguageString {
        language: std::string::String,
        value: std::string::String,
    },
}

/// LSP completion item.
#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::disallowed_types)]
pub struct LspCompletionItem {
    /// The label of this completion item
    pub label: std::string::String,
    /// The kind of this completion item (1=Text, 2=Method, 3=Function, 4=Constructor, 5=Field, 6=Variable, etc.)
    pub kind: Option<u32>,
    /// A human-readable string with additional information
    pub detail: Option<std::string::String>,
    /// A human-readable string that represents a doc-comment
    pub documentation: Option<LspDocumentation>,
    /// A string that should be inserted when selecting this completion
    #[serde(rename = "insertText")]
    pub insert_text: Option<std::string::String>,
    /// The format of the insert text (1=PlainText, 2=Snippet)
    #[serde(rename = "insertTextFormat")]
    pub insert_text_format: Option<u32>,
    /// A string that should be used when filtering a set of completions
    #[serde(rename = "filterText")]
    pub filter_text: Option<std::string::String>,
    /// A string that should be used when comparing this item with other items
    #[serde(rename = "sortText")]
    pub sort_text: Option<std::string::String>,
}

/// LSP documentation - can be string or MarkupContent.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(clippy::disallowed_types)]
pub enum LspDocumentation {
    /// A simple string
    String(std::string::String),
    /// Markup content
    Markup(LspMarkupContent),
}

/// LSP completion list.
#[derive(Debug, Clone, Deserialize)]
pub struct LspCompletionList {
    /// This list is not complete
    #[serde(rename = "isIncomplete")]
    pub is_incomplete: bool,
    /// The completion items
    pub items: Vec<LspCompletionItem>,
}

/// LSP completion response - can be array or list.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LspCompletionResponse {
    /// Array of completion items
    Array(Vec<LspCompletionItem>),
    /// Completion list with metadata
    List(LspCompletionList),
}

impl LspCompletionResponse {
    /// Get items from either variant.
    pub fn items(self) -> Vec<LspCompletionItem> {
        match self {
            LspCompletionResponse::Array(items) => items,
            LspCompletionResponse::List(list) => list.items,
        }
    }
}

/// LSP location link (for definition responses).
#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::disallowed_types)]
pub struct LspLocationLink {
    /// Span of the origin of this link
    #[serde(rename = "originSelectionRange")]
    pub origin_selection_range: Option<LspRange>,
    /// The target resource identifier
    #[serde(rename = "targetUri")]
    pub target_uri: std::string::String,
    /// The full target range
    #[serde(rename = "targetRange")]
    pub target_range: LspRange,
    /// The range that should be selected and revealed
    #[serde(rename = "targetSelectionRange")]
    pub target_selection_range: LspRange,
}

/// LSP definition response - can be location, array, or location links.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LspDefinitionResponse {
    /// A single location
    Scalar(LspLocation),
    /// Array of locations
    Array(Vec<LspLocation>),
    /// Array of location links
    Links(Vec<LspLocationLink>),
}

impl LspDefinitionResponse {
    /// Get locations from any variant.
    pub fn into_locations(self) -> Vec<LspLocation> {
        match self {
            LspDefinitionResponse::Scalar(loc) => vec![loc],
            LspDefinitionResponse::Array(locs) => locs,
            LspDefinitionResponse::Links(links) => links
                .into_iter()
                .map(|link| LspLocation {
                    uri: link.target_uri,
                    range: link.target_selection_range,
                })
                .collect(),
        }
    }
}

/// Result of type checking a document.
#[derive(Debug, Clone, Default)]
pub struct TypeCheckResult {
    /// Diagnostics from type checking
    pub diagnostics: Vec<LspDiagnostic>,
    /// Source map for position translation
    pub source_map: Option<SourceMap>,
}

impl TypeCheckResult {
    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Some(1))
    }

    /// Get error count.
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Some(1))
            .count()
    }

    /// Get warning count.
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Some(2))
            .count()
    }
}

/// Configuration for the Corsa bridge.
#[derive(Debug, Clone)]
pub struct CorsaBridgeConfig {
    /// Path to the native checker executable.
    pub corsa_path: Option<PathBuf>,
    /// Working directory used for module resolution.
    pub working_dir: Option<PathBuf>,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable profiling
    pub enable_profiling: bool,
}

impl Default for CorsaBridgeConfig {
    fn default() -> Self {
        Self {
            corsa_path: None,
            working_dir: None,
            timeout_ms: 30000,
            enable_profiling: false,
        }
    }
}
