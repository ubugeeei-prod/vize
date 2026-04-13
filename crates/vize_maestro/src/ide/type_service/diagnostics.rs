//! Type diagnostic collection.
//!
//! Converts vize_vitrine type check results into LSP diagnostics,
//! including support for the legacy vize_canon type checker and
//! batch type checking via Corsa.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location,
    NumberOrString, Position, Range, Url,
};
use vize_vitrine::{type_check_sfc, TypeCheckOptions, TypeSeverity};

use super::{LspTypeCheckOptions, TypeService};
use crate::server::ServerState;

impl TypeService {
    /// Collect type diagnostics for a document using the strict type checker.
    pub fn collect_diagnostics(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        Self::collect_diagnostics_with_options(state, uri, &LspTypeCheckOptions::default())
    }

    /// Collect type diagnostics with custom options.
    pub fn collect_diagnostics_with_options(
        state: &ServerState,
        uri: &Url,
        lsp_options: &LspTypeCheckOptions,
    ) -> Vec<Diagnostic> {
        let Some(doc) = state.documents.get(uri) else {
            return vec![];
        };

        let content = doc.text();

        // Use vize_vitrine's strict type checker
        let options = TypeCheckOptions {
            filename: uri.path().to_string().into(),
            strict: lsp_options.strict,
            check_props: lsp_options.check_props,
            check_emits: lsp_options.check_emits,
            check_template_bindings: lsp_options.check_template_bindings,
            check_reactivity: lsp_options.check_reactivity,
            check_setup_context: lsp_options.check_setup_context,
            check_invalid_exports: lsp_options.check_invalid_exports,
            check_fallthrough_attrs: lsp_options.check_fallthrough_attrs,
            include_virtual_ts: false,
        };

        let result = type_check_sfc(&content, &options);

        // Convert to LSP diagnostics
        result
            .diagnostics
            .into_iter()
            .map(|diag| {
                let (start_line, start_col) = offset_to_line_col(&content, diag.start as usize);
                let (end_line, end_col) = offset_to_line_col(&content, diag.end as usize);

                // Build related information if present
                let related_information: Option<Vec<DiagnosticRelatedInformation>> = if diag
                    .related
                    .is_empty()
                {
                    None
                } else {
                    Some(
                        diag.related
                            .iter()
                            .map(|rel| {
                                let (rel_start_line, rel_start_col) =
                                    offset_to_line_col(&content, rel.start as usize);
                                let (rel_end_line, rel_end_col) =
                                    offset_to_line_col(&content, rel.end as usize);

                                #[allow(clippy::disallowed_macros)]
                                DiagnosticRelatedInformation {
                                    location: Location {
                                        uri: rel
                                            .filename
                                            .as_ref()
                                            .and_then(|f| Url::parse(&format!("file://{}", f)).ok())
                                            .unwrap_or_else(|| uri.clone()),
                                        range: Range {
                                            start: Position {
                                                line: rel_start_line,
                                                character: rel_start_col,
                                            },
                                            end: Position {
                                                line: rel_end_line,
                                                character: rel_end_col,
                                            },
                                        },
                                    },
                                    #[allow(clippy::disallowed_methods)]
                                    message: rel.message.to_string(),
                                }
                            })
                            .collect(),
                    )
                };

                // Build help message
                #[allow(clippy::disallowed_macros)]
                let message = if let Some(ref help) = diag.help {
                    format!("{}\n\nHelp: {}", diag.message, help)
                } else {
                    #[allow(clippy::disallowed_methods)]
                    diag.message.to_string()
                };

                // Build code description URL
                #[allow(clippy::disallowed_macros)]
                let code_description = diag.code.as_ref().map(|code| CodeDescription {
                    href: Url::parse(&format!(
                        "https://github.com/ubugeeei/vize/wiki/type-errors#{}",
                        code
                    ))
                    .unwrap_or_else(|_| Url::parse("https://github.com/ubugeeei/vize").unwrap()),
                });

                Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_col,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    severity: Some(match diag.severity {
                        TypeSeverity::Error => DiagnosticSeverity::ERROR,
                        TypeSeverity::Warning => DiagnosticSeverity::WARNING,
                        TypeSeverity::Info => DiagnosticSeverity::INFORMATION,
                        TypeSeverity::Hint => DiagnosticSeverity::HINT,
                    }),
                    #[allow(clippy::disallowed_methods)]
                    code: diag.code.map(|c| NumberOrString::String(c.to_string())),
                    code_description,
                    source: Some("vize/types".to_string()),
                    message,
                    related_information,
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Collect diagnostics using the legacy vize_canon type checker.
    /// This is kept for backwards compatibility and can be removed later.
    #[deprecated(
        note = "Use collect_diagnostics which uses the stricter vize_vitrine type checker"
    )]
    pub fn collect_diagnostics_legacy(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        let Some(doc) = state.documents.get(uri) else {
            return vec![];
        };

        let content = doc.text();

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&content, options) else {
            return vec![];
        };

        let Some(ref template) = descriptor.template else {
            return vec![];
        };

        // Build type context from script
        let ctx = Self::build_type_context(&descriptor);

        // Run type checker
        let checker = vize_canon::TypeChecker::new();
        let result = checker.check_template(&template.content, &ctx);

        // Template block offset
        let template_start_line = template.loc.start_line as u32;

        // Convert to LSP diagnostics
        result
            .diagnostics
            .into_iter()
            .map(|diag| {
                let (start_line, start_col) =
                    offset_to_line_col(&template.content, diag.start as usize);
                let (end_line, end_col) = offset_to_line_col(&template.content, diag.end as usize);

                Diagnostic {
                    range: Range {
                        start: Position {
                            line: template_start_line + start_line - 1,
                            character: start_col,
                        },
                        end: Position {
                            line: template_start_line + end_line - 1,
                            character: end_col,
                        },
                    },
                    severity: Some(match diag.severity {
                        vize_canon::TypeSeverity::Error => DiagnosticSeverity::ERROR,
                        vize_canon::TypeSeverity::Warning => DiagnosticSeverity::WARNING,
                    }),
                    code: Some(NumberOrString::Number(diag.code.code() as i32)),
                    source: Some("vize/types".to_string()),
                    #[allow(clippy::disallowed_methods)]
                    message: diag.message.to_string(),
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Run batch type checking on the entire project.
    ///
    /// This uses the Corsa CLI to perform comprehensive TypeScript type checking
    /// on all Vue SFC files in the project. Results are cached for fast access.
    #[cfg(feature = "native")]
    pub fn run_batch_type_check(state: &ServerState) -> Option<super::BatchTypeCheckSummary> {
        let result = state.run_batch_type_check()?;

        Some(super::BatchTypeCheckSummary {
            file_count: state
                .get_batch_checker()
                .map(|c| c.read().file_count())
                .unwrap_or(0),
            error_count: result.error_count(),
            warning_count: result.warning_count(),
            success: result.success,
        })
    }

    /// Check if batch type checking is available.
    #[cfg(feature = "native")]
    pub fn is_batch_available(state: &ServerState) -> bool {
        state.get_batch_checker().is_some()
    }

    /// Get batch type check diagnostics for a specific file.
    #[cfg(feature = "native")]
    pub fn get_batch_diagnostics_for_file(
        state: &ServerState,
        uri: &Url,
    ) -> Vec<vize_canon::BatchDiagnostic> {
        let file_path = match uri.to_file_path() {
            Ok(p) => p,
            Err(_) => return vec![],
        };

        let cache = state.get_batch_cache();
        cache.get_diagnostics(&file_path)
    }

    /// Invalidate batch type check cache.
    /// Should be called when files are modified.
    #[cfg(feature = "native")]
    pub fn invalidate_batch_cache(state: &ServerState) {
        state.invalidate_batch_cache();
    }
}

/// Convert byte offset to (line, column) - line is 1-indexed, column is 0-indexed.
pub(super) fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 1u32;
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
            col += 1;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
}
