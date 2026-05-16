//! Type check service using Corsa.
//!
//! This module provides a high-level API for type checking Vue SFCs
//! using Corsa as the TypeScript type checker backend.

use crate::corsa_bridge::{CorsaBridge, CorsaBridgeError};
use crate::virtual_ts::{generate_virtual_ts_with_offsets, VirtualTsOptions, VirtualTsOutput};
#[allow(clippy::disallowed_types)]
use std::sync::Arc;
use vize_carton::cstr;
use vize_carton::String;

/// Type check service for Vue SFCs.
#[allow(clippy::disallowed_types)]
pub struct TypeCheckService {
    /// The Corsa bridge.
    bridge: Arc<CorsaBridge>,
}

/// Options for type checking.
#[derive(Debug, Clone, Default)]
pub struct TypeCheckServiceOptions {
    /// Project root directory.
    pub project_root: Option<String>,
    /// TypeScript configuration file path.
    pub tsconfig_path: Option<String>,
    /// Whether to check cross-component types.
    pub check_cross_component: bool,
    /// Whether to check template expressions.
    pub check_template: bool,
}

/// Result of type checking a Vue SFC.
#[derive(Debug, Clone, Default)]
pub struct SfcTypeCheckResult {
    /// Diagnostics from Corsa.
    pub diagnostics: Vec<SfcDiagnostic>,
    /// Error count.
    pub error_count: usize,
    /// Warning count.
    pub warning_count: usize,
    /// Generated virtual TypeScript (for debugging).
    pub virtual_ts: Option<String>,
    /// Analysis time in milliseconds.
    pub analysis_time_ms: Option<f64>,
}

/// A diagnostic from type checking.
#[derive(Debug, Clone)]
pub struct SfcDiagnostic {
    /// The diagnostic message.
    pub message: String,
    /// Severity (error, warning).
    pub severity: SfcDiagnosticSeverity,
    /// Start offset in the original SFC.
    pub start: u32,
    /// End offset in the original SFC.
    pub end: u32,
    /// Diagnostic code.
    pub code: Option<String>,
    /// Related information.
    pub related: Vec<SfcRelatedInfo>,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfcDiagnosticSeverity {
    /// Error - must be fixed.
    Error,
    /// Warning - should be fixed.
    Warning,
    /// Information.
    Info,
    /// Hint.
    Hint,
}

/// Related diagnostic information.
#[derive(Debug, Clone)]
pub struct SfcRelatedInfo {
    /// Message.
    pub message: String,
    /// Filename.
    pub filename: Option<String>,
    /// Start offset.
    pub start: u32,
    /// End offset.
    pub end: u32,
}

impl TypeCheckService {
    /// Create a new type check service.
    #[allow(clippy::disallowed_types)]
    pub async fn new() -> Result<Self, CorsaBridgeError> {
        let bridge = CorsaBridge::new();
        bridge.spawn().await?;
        Ok(Self {
            bridge: Arc::new(bridge),
        })
    }

    /// Type check a Vue SFC.
    pub async fn check_sfc(
        &self,
        source: &str,
        filename: &str,
        _options: &TypeCheckServiceOptions,
    ) -> Result<SfcTypeCheckResult, CorsaBridgeError> {
        use std::time::Instant;
        use vize_atelier_core::parser::parse;
        use vize_atelier_sfc::{parse_sfc, SfcParseOptions};
        use vize_carton::Bump;
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let start_time = Instant::now();
        let mut result = SfcTypeCheckResult::default();

        // Parse SFC
        let parse_opts = SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        };

        let descriptor = match parse_sfc(source, parse_opts) {
            Ok(d) => d,
            Err(e) => {
                result.diagnostics.push(SfcDiagnostic {
                    message: cstr!("Failed to parse SFC: {}", e.message),
                    severity: SfcDiagnosticSeverity::Error,
                    start: 0,
                    end: 0,
                    code: Some("parse-error".into()),
                    related: Vec::new(),
                });
                result.error_count = 1;
                return Ok(result);
            }
        };

        // Get script content
        let script_content = descriptor
            .script_setup
            .as_ref()
            .map(|s| s.content.as_ref())
            .or_else(|| descriptor.script.as_ref().map(|s| s.content.as_ref()));

        // Create allocator for template parsing
        let allocator = Bump::new();

        // Create analyzer
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());

        // Analyze script
        let script_offset: u32 = if let Some(ref script_setup) = descriptor.script_setup {
            analyzer.analyze_script_setup(&script_setup.content);
            script_setup.loc.start as u32
        } else if let Some(ref script) = descriptor.script {
            analyzer.analyze_script_plain(&script.content);
            script.loc.start as u32
        } else {
            0
        };

        // Analyze template
        let mut has_template_parse_errors = false;
        let (template_offset, template_ast) = if let Some(ref template) = descriptor.template {
            let template_offset = template.loc.start as u32;
            let (root, errors) = parse(&allocator, &template.content);
            if errors.is_empty() {
                analyzer.analyze_template(&root);
                (template_offset, Some(root))
            } else {
                has_template_parse_errors = true;
                for error in errors {
                    let (start, end) = error
                        .loc
                        .as_ref()
                        .map(|loc| {
                            (
                                template_offset + loc.start.offset,
                                template_offset + loc.end.offset,
                            )
                        })
                        .unwrap_or((template_offset, template_offset));

                    result.diagnostics.push(SfcDiagnostic {
                        message: cstr!("Template parse error: {}", error.message),
                        severity: SfcDiagnosticSeverity::Error,
                        start,
                        end: end.max(start + 1),
                        code: Some("template-parse-error".into()),
                        related: Vec::new(),
                    });
                    result.error_count += 1;
                }
                (template_offset, None)
            }
        } else {
            (0, None)
        };

        let summary = analyzer.finish();

        if has_template_parse_errors {
            result.analysis_time_ms = Some(start_time.elapsed().as_secs_f64() * 1000.0);
            return Ok(result);
        }

        // Generate virtual TypeScript
        let virtual_ts_output = generate_virtual_ts_with_offsets(
            &summary,
            script_content,
            template_ast.as_ref(),
            script_offset,
            template_offset,
            &VirtualTsOptions::default(),
        );

        result.virtual_ts = Some(virtual_ts_output.code.clone());

        // Check with Corsa
        if !virtual_ts_output.code.is_empty() {
            let virtual_uri = cstr!("vize-virtual://{filename}.ts");

            // Open virtual document
            self.bridge
                .open_virtual_document(&virtual_uri, &virtual_ts_output.code)
                .await?;

            // Get diagnostics
            let corsa_result = self.bridge.get_diagnostics(&virtual_uri).await?;

            // Map diagnostics back to original positions
            for diag in corsa_result {
                // Map position from virtual TS to original SFC
                let (start, end) = map_position_to_sfc(
                    &virtual_ts_output,
                    diag.range.start.line,
                    diag.range.start.character,
                    diag.range.end.line,
                    diag.range.end.character,
                    script_offset,
                    template_offset,
                );

                let severity = match diag.severity.unwrap_or(1) {
                    1 => SfcDiagnosticSeverity::Error,
                    2 => SfcDiagnosticSeverity::Warning,
                    3 => SfcDiagnosticSeverity::Info,
                    _ => SfcDiagnosticSeverity::Hint,
                };

                if matches!(severity, SfcDiagnosticSeverity::Error) {
                    result.error_count += 1;
                } else if matches!(severity, SfcDiagnosticSeverity::Warning) {
                    result.warning_count += 1;
                }

                result.diagnostics.push(SfcDiagnostic {
                    message: diag.message.into(),
                    severity,
                    start,
                    end,
                    code: diag.code.map(|c| cstr!("TS{c}")),
                    related: diag
                        .related_information
                        .unwrap_or_default()
                        .into_iter()
                        .map(|r| {
                            // Map related info position from virtual TS to original SFC
                            let (rel_start, rel_end) = map_position_to_sfc(
                                &virtual_ts_output,
                                r.location.range.start.line,
                                r.location.range.start.character,
                                r.location.range.end.line,
                                r.location.range.end.character,
                                script_offset,
                                template_offset,
                            );
                            SfcRelatedInfo {
                                message: r.message.into(),
                                filename: Some(r.location.uri.into()),
                                start: rel_start,
                                end: rel_end,
                            }
                        })
                        .collect(),
                });
            }

            // Close virtual document
            self.bridge.close_virtual_document(&virtual_uri).await?;
        }

        result.analysis_time_ms = Some(start_time.elapsed().as_secs_f64() * 1000.0);
        Ok(result)
    }

    /// Shutdown the type check service.
    pub async fn shutdown(&self) -> Result<(), CorsaBridgeError> {
        self.bridge.shutdown().await
    }
}

/// Convert line and column to offset in the given content.
fn line_col_to_offset(content: &str, line: u32, col: u32) -> u32 {
    let mut offset = 0;
    let mut current_line = 0;

    for (i, ch) in content.char_indices() {
        if current_line == line {
            return (i as u32) + col;
        }
        if ch == '\n' {
            current_line += 1;
        }
        offset = i as u32 + 1;
    }

    offset + col
}

/// Map position from virtual TypeScript to original SFC.
fn map_position_to_sfc(
    virtual_ts: &VirtualTsOutput,
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
    script_offset: u32,
    _template_offset: u32,
) -> (u32, u32) {
    // Convert line/col to offset in generated content
    let gen_start_offset = line_col_to_offset(&virtual_ts.code, start_line, start_char) as usize;
    let gen_end_offset = line_col_to_offset(&virtual_ts.code, end_line, end_char) as usize;

    // Try to find source mapping
    if let Some(mapping) = virtual_ts
        .mappings
        .iter()
        .find(|mapping| mapping.gen_range.contains(&gen_start_offset))
    {
        let src_start =
            mapping.src_range.start as u32 + (gen_start_offset - mapping.gen_range.start) as u32;
        let src_end = if mapping.gen_range.contains(&gen_end_offset) {
            mapping.src_range.start as u32 + (gen_end_offset - mapping.gen_range.start) as u32
        } else {
            mapping.src_range.end as u32
        };
        return (src_start, src_end);
    }

    // Fallback: estimate based on line numbers
    // This is a rough approximation when source map mapping is not found
    let start = script_offset + start_line * 80 + start_char;
    let end = script_offset + end_line * 80 + end_char;
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::{SfcDiagnosticSeverity, TypeCheckServiceOptions};

    #[test]
    fn test_sfc_diagnostic_severity() {
        assert_eq!(SfcDiagnosticSeverity::Error, SfcDiagnosticSeverity::Error);
        assert_ne!(SfcDiagnosticSeverity::Error, SfcDiagnosticSeverity::Warning);
    }

    #[test]
    fn test_type_check_service_options_default() {
        let opts = TypeCheckServiceOptions::default();
        assert!(opts.project_root.is_none());
        assert!(opts.tsconfig_path.is_none());
        assert!(!opts.check_cross_component);
        assert!(!opts.check_template);
    }
}
