//! SFC/JSX frontend and compiler diagnostics.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};
use vize_atelier_sfc::SfcDescriptor;

use crate::server::ServerState;

use super::super::{DiagnosticService, LineIndex, sources};

impl DiagnosticService {
    pub(in crate::ide::diagnostics) fn collect_jsx_diagnostics(
        state: &ServerState,
        uri: &Url,
        content: &str,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        state.ensure_artifact_source(uri, content);
        let Some(syntax) = state.jsx_syntax(uri) else {
            return Vec::new();
        };

        syntax
            .diagnostics
            .iter()
            .map(|diag| {
                let (start_line, start_col) = line_index.line_col(diag.start as usize);
                let (end_line, end_col) = line_index.line_col(diag.end as usize);
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
                        vize_atelier_jsx::Severity::Error => DiagnosticSeverity::ERROR,
                        vize_atelier_jsx::Severity::Warning => DiagnosticSeverity::WARNING,
                    }),
                    source: Some(sources::JSX_COMPILER.to_string()),
                    message: diag.message.to_string(),
                    ..Default::default()
                }
            })
            .collect()
    }

    #[allow(clippy::result_large_err)]
    pub(in crate::ide::diagnostics) fn parse_sfc_for_collect(
        state: &ServerState,
        uri: &Url,
        content: &str,
    ) -> Result<SfcDescriptor<'static>, Diagnostic> {
        state.ensure_artifact_source(uri, content);
        let artifact = state.sfc_descriptor(uri).ok_or_else(|| Diagnostic {
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some(sources::SFC_PARSER.to_string()),
            message: "SFC artifact query failed".to_string(),
            ..Default::default()
        })?;

        match artifact.as_result() {
            Ok(descriptor) => Ok(descriptor.clone()),
            Err(err) => {
                let range = err.loc.as_ref().map_or_else(Range::default, |loc| Range {
                    start: Position {
                        line: loc.start_line.saturating_sub(1) as u32,
                        character: loc.start_column.saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: loc.end_line.saturating_sub(1) as u32,
                        character: loc.end_column.saturating_sub(1) as u32,
                    },
                });
                Err(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some(sources::SFC_PARSER.to_string()),
                    message: err.message.to_string(),
                    ..Default::default()
                })
            }
        }
    }

    pub(in crate::ide::diagnostics) fn collect_template_diagnostics(
        state: &ServerState,
        uri: &Url,
        _content: &str,
        descriptor: &SfcDescriptor<'_>,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        let Some(template) = descriptor.template.as_ref() else {
            return vec![];
        };
        let Some(syntax) = state.sfc_relief(uri) else {
            return vec![];
        };
        let Some(syntax) = syntax.as_ref() else {
            return vec![];
        };

        syntax
            .parse_diagnostics()
            .iter()
            .filter_map(|error| {
                let loc = error.loc.as_ref()?;
                let start = template.loc.start as u32 + loc.start.offset;
                let end = template.loc.start as u32 + loc.end.offset;
                let (start_line, start_character) = line_index.line_col(start as usize);
                let (end_line, end_character) = line_index.line_col(end as usize);
                Some(Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_character,
                        },
                        end: Position {
                            line: end_line,
                            character: end_character,
                        },
                    },
                    severity: Some(if error.is_recoverable() {
                        DiagnosticSeverity::WARNING
                    } else {
                        DiagnosticSeverity::ERROR
                    }),
                    code: Some(NumberOrString::Number(error.code as i32)),
                    source: Some(sources::TEMPLATE_PARSER.to_string()),
                    message: error.message.to_string(),
                    ..Default::default()
                })
            })
            .collect()
    }

    pub(in crate::ide::diagnostics) fn collect_sfc_compile_diagnostics(
        state: &crate::server::ServerState,
        uri: &Url,
        content: &str,
        descriptor: &SfcDescriptor<'_>,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        let Some(script_setup) = descriptor.script_setup.as_ref() else {
            return Vec::new();
        };
        if !script_setup_has_validator_candidates(&script_setup.content) {
            return Vec::new();
        }
        let Some(syntax) = state.sfc_script_syntax(uri) else {
            return Vec::new();
        };
        let Err(err) = syntax.validate_script_setup_semantics(content) else {
            return Vec::new();
        };

        let range = err.loc.as_ref().map_or_else(
            || {
                let offset = vize_canon::sfc_block_fallback_offset(descriptor)
                    .map_or(0, |(offset, _)| offset);
                let (line, character) = line_index.line_col(offset);
                Range {
                    start: Position { line, character },
                    end: Position { line, character },
                }
            },
            |loc| Range {
                start: Position {
                    line: (loc.start_line as u32).saturating_sub(1),
                    character: (loc.start_column as u32).saturating_sub(1),
                },
                end: Position {
                    line: (loc.end_line as u32).saturating_sub(1),
                    character: (loc.end_column as u32).saturating_sub(1),
                },
            },
        );
        let message = err.code.as_deref().map_or_else(
            || err.message.to_string(),
            |code| format!("[{code}] {}", err.message),
        );
        vec![Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: err
                .code
                .as_deref()
                .map(|code| NumberOrString::String(code.to_string())),
            source: Some(sources::SFC_COMPILER.to_string()),
            message,
            ..Default::default()
        }]
    }
}

fn script_setup_has_validator_candidates(content: &str) -> bool {
    content.contains("defineProps<") && content.contains("= defineProps")
}
