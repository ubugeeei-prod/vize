//! Mapping Corsa virtual-document diagnostics back to the host SFC.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};
use vize_carton::FxHashSet;

use super::super::{VirtualTsResult, sources};
use super::mapping::{
    line_character_to_byte_offset, map_diagnostic_with_source_mappings, source_offset_to_position,
};
use super::message::rewrite_corsa_message;
use super::script_fallback::ScriptFallback;

// Both collectors surface bridge failures to the caller instead of mapping
// them to an empty diagnostic list: `collect_corsa_diagnostics` uses the
// error to distinguish a dead backend process (retire + respawn + retry,
// #3240) from an ordinary per-request failure.
pub(super) async fn collect_virtual_result_diagnostics(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    host_uri: &Url,
    content: &str,
    virtual_name: String,
    virtual_result: VirtualTsResult,
) -> Result<Vec<Diagnostic>, vize_canon::CorsaBridgeError> {
    let virtual_uri = bridge
        .open_or_update_virtual_document(&virtual_name, &virtual_result.code)
        .await?;

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
) -> Result<Vec<Diagnostic>, vize_canon::CorsaBridgeError> {
    let virtual_ts = &virtual_result.code;
    let user_code_start_line = virtual_result.user_code_start_line;
    let sfc_script_start_line = virtual_result.sfc_script_start_line;
    let template_scope_start_line = virtual_result.template_scope_start_line;
    let line_mappings = &virtual_result.line_mappings;
    let source_mappings = &virtual_result.source_mappings;
    // Positions the source map cannot place are guessed by line arithmetic and
    // must stay inside the authored document (#3299). A trailing newline does
    // not open a line an editor can render a range on, so it is not counted.
    let script_fallback = ScriptFallback {
        user_code_start_line,
        sfc_script_start_line,
        skipped_import_lines: virtual_result.skipped_import_lines,
        authored_line_count: content.lines().count() as u32,
    };
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
    let corsa_diags = bridge.get_diagnostics(&virtual_uri).await?;

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
            if is_generated_vue_ts_import_extension_diagnostic(virtual_ts, &diag) {
                tracing::debug!("skipping generated .vue.ts import extension diagnostic");
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

            let (start_line, end_line, start_char, end_char) =
                if let Some(mapped_range) = mapped_range {
                    mapped_range
                } else if is_template_error {
                    let virtual_line = diag.range.start.line as usize;
                    let mapping = (0..=10)
                        .find_map(|offset| line_mappings.get(virtual_line + offset)?.as_ref());

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
                } else if let Some((start, end)) =
                    script_fallback.guess_range(diag.range.start.line, diag.range.end.line)
                {
                    (
                        start,
                        end,
                        diag.range.start.character.saturating_sub(2),
                        diag.range.end.character.saturating_sub(2),
                    )
                } else {
                    tracing::debug!(
                        "skipping unplaceable script diagnostic at virtual line {}: {}",
                        diag.range.start.line,
                        &diag.message[..diag.message.len().min(50)]
                    );
                    return None;
                };

            if is_authored_vue_import_extension_diagnostic(
                content, &diag, start_line, start_char, end_line, end_char,
            ) {
                tracing::debug!("skipping mapped .vue import extension diagnostic");
                return None;
            }

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
                message: rewrite_corsa_message(&diag.message, content),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();

    Ok(deduplicate_diagnostics(mapped_diagnostics))
}

pub(super) fn deduplicate_diagnostics(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let mut seen = FxHashSet::default();
    diagnostics.retain(|diagnostic| match serde_json::to_vec(diagnostic) {
        Ok(key) => seen.insert(key),
        Err(error) => {
            tracing::warn!("failed to serialize diagnostic deduplication key: {error}");
            true
        }
    });
    diagnostics
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

fn is_generated_vue_ts_import_extension_diagnostic(
    virtual_ts: &str,
    diagnostic: &vize_canon::corsa_bridge::LspDiagnostic,
) -> bool {
    if !is_ts5097_import_extension_diagnostic(diagnostic) {
        return false;
    }

    let Some(start) = line_character_to_byte_offset(
        virtual_ts,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    ) else {
        return false;
    };
    let Some(end) = line_character_to_byte_offset(
        virtual_ts,
        diagnostic.range.end.line,
        diagnostic.range.end.character,
    ) else {
        return false;
    };
    virtual_ts
        .get(start..end)
        .is_some_and(|range| range.contains(".vue.ts") || range.contains(".vue.tsx"))
}

fn is_authored_vue_import_extension_diagnostic(
    content: &str,
    diagnostic: &vize_canon::corsa_bridge::LspDiagnostic,
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
) -> bool {
    if !is_ts5097_import_extension_diagnostic(diagnostic) {
        return false;
    }
    let Some(start) = line_character_to_byte_offset(content, start_line, start_character) else {
        return false;
    };
    let Some(end) = line_character_to_byte_offset(content, end_line, end_character) else {
        return false;
    };
    content
        .get(start..end)
        .is_some_and(|range| range.contains(".vue"))
}

fn is_ts5097_import_extension_diagnostic(
    diagnostic: &vize_canon::corsa_bridge::LspDiagnostic,
) -> bool {
    diagnostic.code.as_ref().is_some_and(|code| match code {
        serde_json::Value::Number(number) => number.as_i64() == Some(5097),
        serde_json::Value::String(code) => matches!(code.as_str(), "5097" | "TS5097"),
        _ => false,
    }) && diagnostic.message.contains("allowImportingTsExtensions")
}

pub(in crate::ide) fn corsa_diagnostic_code(code: serde_json::Value) -> NumberOrString {
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
        corsa_diagnostic_code, deduplicate_diagnostics,
        is_authored_vue_import_extension_diagnostic,
        is_generated_vue_ts_import_extension_diagnostic, is_inferred_implicit_any_suggestion,
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
    fn generated_vue_ts_import_extension_diagnostics_are_suppressed() {
        let virtual_ts = "import Child from './Child.vue.ts';\nimport plain from './plain.ts';\n";
        let diagnostic = |start, end| {
            LspDiagnostic {
                range: LspRange {
                    start: LspPosition {
                        line: 0,
                        character: start,
                    },
                    end: LspPosition {
                        line: 0,
                        character: end,
                    },
                },
                severity: Some(1),
                code: Some(serde_json::json!(5097)),
                source: Some("ts".into()),
                message:
                    "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled."
                        .into(),
                related_information: None,
            }
        };

        assert!(is_generated_vue_ts_import_extension_diagnostic(
            virtual_ts,
            &diagnostic(18, 34),
        ));

        let authored_ts_import = LspDiagnostic {
            range: LspRange {
                start: LspPosition {
                    line: 1,
                    character: 18,
                },
                end: LspPosition {
                    line: 1,
                    character: 30,
                },
            },
            ..diagnostic(18, 34)
        };
        assert!(!is_generated_vue_ts_import_extension_diagnostic(
            virtual_ts,
            &authored_ts_import,
        ));
    }

    #[test]
    fn mapped_authored_vue_import_extension_diagnostics_are_suppressed() {
        let content = "<script setup lang=\"ts\">\nimport Child from './Child.vue'\nimport plain from './plain.ts'\n</script>\n";
        let diagnostic = LspDiagnostic {
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
            severity: Some(1),
            code: Some(serde_json::json!("TS5097")),
            source: Some("ts".into()),
            message:
                "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled."
                    .into(),
            related_information: None,
        };

        assert!(is_authored_vue_import_extension_diagnostic(
            content,
            &diagnostic,
            1,
            18,
            1,
            31,
        ));
        assert!(!is_authored_vue_import_extension_diagnostic(
            content,
            &diagnostic,
            2,
            18,
            2,
            30,
        ));
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
        let distinct_data = Diagnostic {
            data: Some(serde_json::json!({ "origin": "second-pass" })),
            ..original.clone()
        };

        assert_eq!(
            deduplicate_diagnostics(vec![
                original.clone(),
                distinct.clone(),
                original.clone(),
                distinct.clone(),
                distinct_data.clone(),
                distinct_data.clone(),
            ]),
            vec![original, distinct, distinct_data],
        );
    }
}
