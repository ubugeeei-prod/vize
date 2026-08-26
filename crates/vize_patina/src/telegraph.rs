//! Patina emitters built on Carton's Telegraph message fan-out.
//!
//! Patina keeps lint-specific formatting here while Carton owns the generic
//! `Telegraph` and `Emitter` routing primitives. This lets other Vize crates
//! reuse the delivery system without depending on Patina diagnostics.
//!
//! ## Name Origin
//!
//! A **telegraph** is a communication system that transmits messages over
//! long distances. Just as telegraphs revolutionized how people communicated
//! across distances, `Telegraph` delivers lint diagnostics to various receivers
//! - terminal output, language servers, or other tools.
//!
//! ## Architecture
//!
//! ```text
//! LintResult --> Telegraph --> Emitter --> Destination
//!                              |
//!                              +-- TextEmitter  --> stdout (rich terminal)
//!                              +-- JsonEmitter  --> JSON format
//!                              +-- FormatEmitter --> ansi/plain/stylish/markdown/html/agent reports
//!                              +-- LspEmitter   --> LSP diagnostics
//!                              +-- OxlintBridge --> oxlint (future)
//! ```

#![allow(clippy::disallowed_macros)]

use crate::diagnostic::{HelpRenderTarget, Severity, render_help};
use crate::linter::LintResult;
use crate::output::{OutputFormat, format_results};
use vize_s0::String;
use vize_s0::ToCompactString;
pub use vize_s0::telegraph::Emitter;
use vize_s0::telegraph::Telegraph as CartonTelegraph;

/// A Patina lint result and its source text as a Telegraph message.
#[derive(Debug, Clone)]
pub struct LintTransmission {
    /// Lint diagnostics for one file.
    pub result: LintResult,
    /// Source text for location-aware renderers.
    pub source: String,
}

impl LintTransmission {
    /// Create a lint transmission message.
    pub fn new(result: LintResult, source: impl Into<String>) -> Self {
        Self {
            result,
            source: source.into(),
        }
    }
}

/// Telegraph coordinates the delivery of lint results to emitters.
///
/// It acts as a dispatcher, routing diagnostics to the appropriate
/// output channels based on configuration.
pub struct Telegraph {
    inner: CartonTelegraph<LintTransmission, String>,
}

impl Telegraph {
    /// Create a new Telegraph with no emitters
    pub fn new() -> Self {
        Self {
            inner: CartonTelegraph::new(),
        }
    }

    /// Create Telegraph with the default text emitter
    pub fn with_text() -> Self {
        let mut telegraph = Self::new();
        telegraph.add_emitter(TextEmitter::default());
        telegraph
    }

    /// Create Telegraph with JSON emitter
    pub fn with_json() -> Self {
        let mut telegraph = Self::new();
        telegraph.add_emitter(JsonEmitter);
        telegraph
    }

    /// Create Telegraph with a single output-format emitter.
    pub fn with_format(format: OutputFormat) -> Self {
        let mut telegraph = Self::new();
        match format {
            OutputFormat::Text => telegraph.add_emitter(TextEmitter::default()),
            OutputFormat::Json => telegraph.add_emitter(JsonEmitter),
            _ => telegraph.add_emitter(FormatEmitter::new(format)),
        }
        telegraph
    }

    /// Add an emitter to the telegraph
    pub fn add_emitter<E>(&mut self, emitter: E)
    where
        E: Emitter<LintTransmission, Output = String> + 'static,
    {
        self.inner.add_emitter(emitter);
    }

    /// Number of registered emitters.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether no emitters are registered.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Transmit a single result through all emitters
    pub fn transmit(&self, result: &LintResult, source: &str) -> Vec<String> {
        let transmission = LintTransmission::new(result.clone(), source.to_compact_string());
        self.inner.transmit(&transmission)
    }

    /// Transmit multiple results through all emitters
    pub fn transmit_all(&self, results: &[(LintResult, String)]) -> Vec<String> {
        let transmissions: Vec<_> = results
            .iter()
            .map(|(result, source)| LintTransmission::new(result.clone(), source.clone()))
            .collect();
        self.inner.transmit_all(&transmissions)
    }
}

impl Default for Telegraph {
    fn default() -> Self {
        Self::with_text()
    }
}

/// Text emitter for rich terminal output (oxlint-style)
#[derive(Default)]
pub struct TextEmitter {
    /// Whether to use colors in output
    pub colors: bool,
}

impl TextEmitter {
    pub fn new(colors: bool) -> Self {
        Self { colors }
    }
}

impl TextEmitter {
    fn emit_summary(&self, transmissions: &[LintTransmission]) -> String {
        let total_errors: usize = transmissions
            .iter()
            .map(|transmission| transmission.result.error_count)
            .sum();
        let total_warnings: usize = transmissions
            .iter()
            .map(|transmission| transmission.result.warning_count)
            .sum();
        let file_count = transmissions.len();

        if total_errors == 0 && total_warnings == 0 {
            return String::default();
        }

        format!(
            "\nFound {} error{} and {} warning{} in {} file{}.\n",
            total_errors,
            if total_errors == 1 { "" } else { "s" },
            total_warnings,
            if total_warnings == 1 { "" } else { "s" },
            file_count,
            if file_count == 1 { "" } else { "s" },
        )
        .into()
    }
}

impl Emitter<LintTransmission> for TextEmitter {
    type Output = String;

    fn name(&self) -> &'static str {
        "text"
    }

    fn emit(&self, transmission: &LintTransmission) -> String {
        let files = vec![(
            transmission.result.filename.clone(),
            transmission.source.clone(),
        )];
        format_results(
            std::slice::from_ref(&transmission.result),
            &files,
            OutputFormat::Text,
        )
    }

    fn emit_all(&self, transmissions: &[LintTransmission]) -> String {
        let mut output = String::default();
        for transmission in transmissions {
            output.push_str(&self.emit(transmission));
        }
        output.push_str(&self.emit_summary(transmissions));
        output
    }
}

/// JSON emitter for machine-readable output
pub struct JsonEmitter;

impl Emitter<LintTransmission> for JsonEmitter {
    type Output = String;

    fn name(&self) -> &'static str {
        "json"
    }

    fn emit(&self, transmission: &LintTransmission) -> String {
        let files = vec![(
            transmission.result.filename.clone(),
            transmission.source.clone(),
        )];
        format_results(
            std::slice::from_ref(&transmission.result),
            &files,
            OutputFormat::Json,
        )
    }

    fn emit_all(&self, transmissions: &[LintTransmission]) -> String {
        emit_format_all(transmissions, OutputFormat::Json)
    }
}

/// Generic output-format emitter for whole-report transforms.
pub struct FormatEmitter {
    format: OutputFormat,
}

impl FormatEmitter {
    pub const fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub const fn format(&self) -> OutputFormat {
        self.format
    }
}

impl Emitter<LintTransmission> for FormatEmitter {
    type Output = String;

    fn name(&self) -> &'static str {
        self.format.as_str()
    }

    fn emit(&self, transmission: &LintTransmission) -> String {
        let files = vec![(
            transmission.result.filename.clone(),
            transmission.source.clone(),
        )];
        format_results(
            std::slice::from_ref(&transmission.result),
            &files,
            self.format,
        )
    }

    fn emit_all(&self, transmissions: &[LintTransmission]) -> String {
        emit_format_all(transmissions, self.format)
    }
}

fn emit_format_all(transmissions: &[LintTransmission], format: OutputFormat) -> String {
    let lint_results: Vec<_> = transmissions
        .iter()
        .map(|transmission| transmission.result.clone())
        .collect();
    let sources: Vec<_> = transmissions
        .iter()
        .map(|transmission| {
            (
                transmission.result.filename.clone(),
                transmission.source.clone(),
            )
        })
        .collect();

    format_results(&lint_results, &sources, format)
}

/// LSP emitter for Language Server Protocol diagnostics.
///
/// Converts lint diagnostics to LSP-compatible format for IDE integration.
pub struct LspEmitter;

/// LSP-compatible diagnostic representation
#[derive(Debug, Clone, serde::Serialize)]
pub struct LspDiagnostic {
    /// The range at which the diagnostic applies
    pub range: LspRange,
    /// The diagnostic's severity (1 = Error, 2 = Warning, 3 = Info, 4 = Hint)
    pub severity: u8,
    /// A human-readable message
    pub message: String,
    /// The source of this diagnostic (e.g., "vize-patina")
    pub source: String,
    /// The diagnostic's code (rule name)
    pub code: String,
}

/// LSP-compatible range
#[derive(Debug, Clone, serde::Serialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

/// LSP-compatible position
#[derive(Debug, Clone, serde::Serialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

impl LspEmitter {
    /// Convert a LintResult to LSP diagnostics without source context.
    ///
    /// This is a degraded variant that cannot resolve line/column from byte
    /// offsets and therefore always reports `line: 0` with the raw byte offset
    /// in `character`. Prefer [`Self::to_lsp_diagnostics_with_source`] whenever
    /// the source text is available; the `Emitter<LintTransmission>` impl
    /// already does so.
    pub fn to_lsp_diagnostics(result: &LintResult) -> Vec<LspDiagnostic> {
        result
            .diagnostics
            .iter()
            .map(|d| LspDiagnostic {
                range: LspRange {
                    start: LspPosition {
                        // TODO: Convert byte offset to line/column using source
                        line: 0,
                        character: d.start,
                    },
                    end: LspPosition {
                        line: 0,
                        character: d.end,
                    },
                },
                severity: match d.severity {
                    Severity::Error => 1,
                    Severity::Warning => 2,
                },
                message: if let Some(help) = &d.help {
                    format!(
                        "{}\n{}",
                        d.message,
                        render_help(help, HelpRenderTarget::PlainText)
                    )
                    .into()
                } else {
                    d.message.to_compact_string()
                },
                source: "vize-patina".to_compact_string(),
                code: d.rule_name.to_compact_string(),
            })
            .collect()
    }

    /// Convert a LintResult to LSP diagnostics with accurate line/column info
    pub fn to_lsp_diagnostics_with_source(result: &LintResult, source: &str) -> Vec<LspDiagnostic> {
        result
            .diagnostics
            .iter()
            .map(|d| {
                let (start_line, start_col) = offset_to_line_col(source, d.start as usize);
                let (end_line, end_col) = offset_to_line_col(source, d.end as usize);

                LspDiagnostic {
                    range: LspRange {
                        start: LspPosition {
                            line: start_line,
                            character: start_col,
                        },
                        end: LspPosition {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    severity: match d.severity {
                        Severity::Error => 1,
                        Severity::Warning => 2,
                    },
                    message: if let Some(help) = &d.help {
                        format!(
                            "{}\n{}",
                            d.message,
                            render_help(help, HelpRenderTarget::PlainText)
                        )
                        .into()
                    } else {
                        d.message.to_compact_string()
                    },
                    source: "vize-patina".to_compact_string(),
                    code: d.rule_name.to_compact_string(),
                }
            })
            .collect()
    }
}

/// Convert byte offset to (line, column) - both 0-indexed for LSP
fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut current_offset = 0;

    for ch in source.chars() {
        if current_offset >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
}

impl Emitter<LintTransmission> for LspEmitter {
    type Output = String;

    fn name(&self) -> &'static str {
        "lsp"
    }

    fn emit(&self, transmission: &LintTransmission) -> String {
        let diagnostics =
            Self::to_lsp_diagnostics_with_source(&transmission.result, &transmission.source);
        serde_json::to_string_pretty(&diagnostics)
            .unwrap_or_default()
            .into()
    }

    fn emit_all(&self, transmissions: &[LintTransmission]) -> String {
        let diagnostics: Vec<_> = transmissions
            .iter()
            .flat_map(|transmission| {
                Self::to_lsp_diagnostics_with_source(&transmission.result, &transmission.source)
            })
            .collect();
        serde_json::to_string_pretty(&diagnostics)
            .unwrap_or_default()
            .into()
    }
}

/// Future: Bridge to oxlint plugin system
///
/// This will be implemented when oxlint provides plugin APIs.
/// The bridge will allow vize_patina rules to be used as oxlint plugins.
#[doc(hidden)]
pub struct OxlintBridge {
    // Reserved for future implementation
}

#[cfg(test)]
mod tests;
