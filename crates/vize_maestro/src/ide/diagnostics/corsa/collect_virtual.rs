//! Mapping Corsa virtual-document diagnostics back to the host SFC.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};

use super::super::{VirtualTsResult, sources};
use super::mapping::{map_diagnostic_with_source_mappings, source_offset_to_position};
use super::message::rewrite_corsa_message;

pub(super) async fn collect_virtual_result_diagnostics(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    host_uri: &Url,
    content: &str,
    virtual_name: String,
    virtual_result: VirtualTsResult,
) -> Vec<Diagnostic> {
    let virtual_uri = match bridge
        .open_or_update_virtual_document(&virtual_name, &virtual_result.code)
        .await
    {
        Ok(uri) => uri,
        Err(e) => {
            tracing::warn!("failed to open/update virtual document: {}", e);
            return vec![];
        }
    };

    collect_synced_virtual_result_diagnostics(
        bridge,
        host_uri,
        content,
        virtual_uri.to_string(),
        virtual_result,
    )
    .await
}

pub(super) async fn collect_synced_virtual_result_diagnostics(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    _host_uri: &Url,
    content: &str,
    virtual_uri: String,
    virtual_result: VirtualTsResult,
) -> Vec<Diagnostic> {
    let virtual_ts = &virtual_result.code;
    let user_code_start_line = virtual_result.user_code_start_line;
    let sfc_script_start_line = virtual_result.sfc_script_start_line;
    let template_scope_start_line = virtual_result.template_scope_start_line;
    let line_mappings = &virtual_result.line_mappings;
    let source_mappings = &virtual_result.source_mappings;
    tracing::info!(
        "generated virtual ts ({} bytes), user_code_start={}, sfc_script_start={}, template_scope_start={}, mappings_count={}",
        virtual_ts.len(),
        user_code_start_line,
        sfc_script_start_line,
        template_scope_start_line,
        line_mappings.iter().filter(|m| m.is_some()).count()
    );

    tracing::info!(
        "waiting for diagnostics from corsa bridge for {}",
        virtual_uri
    );
    let Ok(corsa_diags) = bridge.get_diagnostics(&virtual_uri).await else {
        tracing::warn!("failed to get diagnostics from corsa");
        return vec![];
    };

    tracing::info!(
        "corsa returned {} raw diagnostics for {}",
        corsa_diags.len(),
        virtual_uri
    );

    for (i, diag) in corsa_diags.iter().enumerate() {
        tracing::info!(
            "  raw diag[{}]: line {}-{}, message: {}",
            i,
            diag.range.start.line,
            diag.range.end.line,
            &diag.message[..diag.message.len().min(100)]
        );
    }

    let mapped_diagnostics = corsa_diags
        .into_iter()
        .filter_map(|diag| {
            if is_inferred_implicit_any_suggestion(&diag) {
                tracing::debug!("skipping TS7044 inference suggestion");
                return None;
            }

            let is_unused_warning = diag.message.contains("is declared but")
                && (diag.message.contains("never read") || diag.message.contains("never used"));
            let is_internal_var = diag.message.contains("'__")
                || diag.message.contains("'$event'")
                || diag.message.contains("'$attrs'")
                || diag.message.contains("'$slots'")
                || diag.message.contains("'$refs'")
                || diag.message.contains("'$emit'");

            if is_unused_warning && is_internal_var {
                tracing::debug!(
                    "skipping internal variable warning: {}",
                    &diag.message[..diag.message.len().min(80)]
                );
                return None;
            }

            let mapped_range = map_diagnostic_with_source_mappings(
                virtual_ts,
                content,
                source_mappings,
                &virtual_result.import_source_map,
                diag.range.start.line,
                diag.range.start.character,
                diag.range.end.line,
                diag.range.end.character,
            );

            let is_template_error = diag.range.start.line >= template_scope_start_line;

            let (start_line, end_line, start_char, end_char) = if let Some(mapped_range) =
                mapped_range
            {
                mapped_range
            } else if is_template_error {
                let virtual_line = diag.range.start.line as usize;
                let mapping =
                    (0..=10).find_map(|offset| line_mappings.get(virtual_line + offset)?.as_ref());

                if let Some(src_mapping) = mapping {
                    let (start_line, start_col) =
                        source_offset_to_position(content, src_mapping.start as usize);
                    let (end_line, end_col) =
                        source_offset_to_position(content, src_mapping.end as usize);
                    (start_line, end_line, start_col, end_col)
                } else {
                    tracing::debug!(
                        "skipping unmapped template error at line {}: {}",
                        diag.range.start.line,
                        &diag.message[..diag.message.len().min(50)]
                    );
                    return None;
                }
            } else {
                if diag.range.start.line < user_code_start_line {
                    tracing::debug!(
                        "skipping preamble diagnostic at line {} (user code starts at {}): {}",
                        diag.range.start.line,
                        user_code_start_line,
                        &diag.message[..diag.message.len().min(50)]
                    );
                    return None;
                }

                let user_code_offset = diag.range.start.line.saturating_sub(user_code_start_line);
                let user_code_offset_end = diag.range.end.line.saturating_sub(user_code_start_line);
                let skipped_lines = virtual_result.skipped_import_lines;
                let start =
                    (sfc_script_start_line.saturating_sub(1)) + user_code_offset + skipped_lines;
                let end = (sfc_script_start_line.saturating_sub(1))
                    + user_code_offset_end
                    + skipped_lines;
                (
                    start,
                    end,
                    diag.range.start.character.saturating_sub(2),
                    diag.range.end.character.saturating_sub(2),
                )
            };

            Some(Diagnostic {
                range: Range {
                    start: Position {
                        line: start_line,
                        character: start_char,
                    },
                    end: Position {
                        line: end_line,
                        character: end_char,
                    },
                },
                severity: diag.severity.map(|s| match s {
                    1 => DiagnosticSeverity::ERROR,
                    2 => DiagnosticSeverity::WARNING,
                    3 => DiagnosticSeverity::INFORMATION,
                    _ => DiagnosticSeverity::HINT,
                }),
                code: diag.code.map(corsa_diagnostic_code),
                source: Some(sources::TYPE_CHECKER.to_string()),
                message: rewrite_corsa_message(&diag.message),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();

    deduplicate_diagnostics(mapped_diagnostics)
}

fn deduplicate_diagnostics(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let mut unique = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics {
        if !unique.contains(&diagnostic) {
            unique.push(diagnostic);
        }
    }
    unique
}

fn is_inferred_implicit_any_suggestion(
    diagnostic: &vize_canon::corsa_bridge::LspDiagnostic,
) -> bool {
    diagnostic.severity == Some(4)
        && diagnostic.code.as_ref().is_some_and(|code| match code {
            serde_json::Value::Number(number) => number.as_i64() == Some(7044),
            serde_json::Value::String(code) => matches!(code.as_str(), "7044" | "TS7044"),
            _ => false,
        })
}

fn corsa_diagnostic_code(code: serde_json::Value) -> NumberOrString {
    match code {
        serde_json::Value::Number(number) => number.as_i64().map_or_else(
            || NumberOrString::String(number.to_string()),
            |value| {
                i32::try_from(value).map_or_else(
                    |_| NumberOrString::String(number.to_string()),
                    NumberOrString::Number,
                )
            },
        ),
        serde_json::Value::String(code) => NumberOrString::String(code),
        other => NumberOrString::String(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        corsa_diagnostic_code, deduplicate_diagnostics, is_inferred_implicit_any_suggestion,
    };
    use tower_lsp::lsp_types::{Diagnostic, NumberOrString, Position, Range};
    use vize_canon::corsa_bridge::{LspDiagnostic, LspPosition, LspRange};

    #[test]
    fn corsa_diagnostic_codes_preserve_lsp_number_and_string_shapes() {
        assert_eq!(
            corsa_diagnostic_code(serde_json::json!(2322)),
            NumberOrString::Number(2322),
        );
        assert_eq!(
            corsa_diagnostic_code(serde_json::json!("TS2322")),
            NumberOrString::String("TS2322".to_string()),
        );
    }

    #[test]
    fn only_ts7044_hints_are_suppressed() {
        let diagnostic = |severity, code| LspDiagnostic {
            range: LspRange {
                start: LspPosition {
                    line: 0,
                    character: 0,
                },
                end: LspPosition {
                    line: 0,
                    character: 1,
                },
            },
            severity,
            code,
            source: Some("ts".into()),
            message: "diagnostic".into(),
            related_information: None,
        };

        assert!(is_inferred_implicit_any_suggestion(&diagnostic(
            Some(4),
            Some(serde_json::json!(7044)),
        )));
        assert!(is_inferred_implicit_any_suggestion(&diagnostic(
            Some(4),
            Some(serde_json::json!("TS7044")),
        )));
        assert!(!is_inferred_implicit_any_suggestion(&diagnostic(
            Some(1),
            Some(serde_json::json!(7044)),
        )));
        assert!(!is_inferred_implicit_any_suggestion(&diagnostic(
            Some(4),
            Some(serde_json::json!(7043)),
        )));
        assert!(!is_inferred_implicit_any_suggestion(&diagnostic(
            None, None,
        )));
    }

    #[test]
    fn exact_diagnostics_are_stably_deduplicated() {
        let original = Diagnostic {
            range: Range {
                start: Position {
                    line: 1,
                    character: 19,
                },
                end: Position {
                    line: 1,
                    character: 30,
                },
            },
            severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::Number(2304)),
            source: Some("vize/types".into()),
            message: "Cannot find name 'missingList'.".into(),
            ..Default::default()
        };
        let distinct = Diagnostic {
            message: "Cannot find name 'anotherBinding'.".into(),
            ..original.clone()
        };

        assert_eq!(
            deduplicate_diagnostics(vec![
                original.clone(),
                distinct.clone(),
                original.clone(),
                distinct.clone(),
            ]),
            vec![original, distinct],
        );
    }
}
