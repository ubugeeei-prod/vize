//! Type definitions for SFC type checking results.
//!
//! Contains severity levels, diagnostic types, result containers,
//! and configuration options for the SFC type checker.

use serde::Serialize;
use vize_carton::String;

/// Type diagnostic severity.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SfcTypeSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Type diagnostic representing a type-related issue.
#[derive(Debug, Clone, Serialize)]
pub struct SfcTypeDiagnostic {
    /// Severity of the diagnostic
    pub severity: SfcTypeSeverity,
    /// Human-readable message
    pub message: String,
    /// Start offset in source
    pub start: u32,
    /// End offset in source
    pub end: u32,
    /// Optional error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Optional help text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// Related locations (for multi-file issues)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<SfcRelatedLocation>,
}

/// Related location for diagnostics.
#[derive(Debug, Clone, Serialize)]
pub struct SfcRelatedLocation {
    pub message: String,
    pub start: u32,
    pub end: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Type checking result.
#[derive(Debug, Clone, Serialize)]
pub struct SfcTypeCheckResult {
    /// List of diagnostics
    pub diagnostics: Vec<SfcTypeDiagnostic>,
    /// Generated virtual TypeScript (for debugging/IDE integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_ts: Option<String>,
    /// Error count
    pub error_count: usize,
    /// Warning count
    pub warning_count: usize,
    /// Analysis time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis_time_ms: Option<f64>,
}

impl SfcTypeCheckResult {
    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            diagnostics: Vec::new(),
            virtual_ts: None,
            error_count: 0,
            warning_count: 0,
            analysis_time_ms: None,
        }
    }

    /// Add a diagnostic.
    pub fn add_diagnostic(&mut self, diagnostic: SfcTypeDiagnostic) {
        match diagnostic.severity {
            SfcTypeSeverity::Error => self.error_count += 1,
            SfcTypeSeverity::Warning => self.warning_count += 1,
            _ => {}
        }
        self.diagnostics.push(diagnostic);
    }

    /// Check if there are errors.
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

/// Type checking options.
#[derive(Debug, Clone, Default)]
pub struct SfcTypeCheckOptions {
    /// Filename for error reporting
    pub filename: String,
    /// Whether to include virtual TypeScript in output
    pub include_virtual_ts: bool,
    /// Whether to check props types
    pub check_props: bool,
    /// Whether to check emits types
    pub check_emits: bool,
    /// Whether to check template bindings
    pub check_template_bindings: bool,
    /// Whether to check reactivity loss patterns
    pub check_reactivity: bool,
    /// Whether to check setup context violations
    pub check_setup_context: bool,
    /// Whether to check invalid exports in `<script setup>`
    pub check_invalid_exports: bool,
    /// Whether to check fallthrough attrs with multi-root
    pub check_fallthrough_attrs: bool,
    /// Strict mode - report more potential issues
    pub strict: bool,
}

impl SfcTypeCheckOptions {
    /// Create default options.
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            include_virtual_ts: false,
            check_props: true,
            check_emits: true,
            check_template_bindings: true,
            check_reactivity: true,
            check_setup_context: true,
            check_invalid_exports: true,
            check_fallthrough_attrs: true,
            strict: false,
        }
    }

    /// Enable strict mode.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Include virtual TypeScript in output.
    pub fn with_virtual_ts(mut self) -> Self {
        self.include_virtual_ts = true;
        self
    }
}
