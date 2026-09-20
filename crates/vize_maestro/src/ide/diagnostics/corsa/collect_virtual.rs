//! Mapping Corsa virtual-document diagnostics back to the host SFC.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};
use vize_s0::FxHashSet;
use vize_s0::line_index::LineBreaks;

use super::super::{VirtualTsResult, sources};
use super::log_preview::log_preview;
use super::mapping::{line_character_to_byte_offset, map_diagnostic_with_source_mappings};
use super::message::rewrite_corsa_message;

pub(super) async fn collect_synced_virtual_result_diagnostics(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    _host_uri: &Url,
    content: &str,
    virtual_uri: String,
    virtual_result: VirtualTsResult,
) -> Result<Vec<Diagnostic>, vize_canon::CorsaBridgeError> {
    let virtual_ts = &virtual_result.code;
    let source_mappings = &virtual_result.source_mappings;
    tracing::info!(
        "generated virtual ts ({} bytes), mappings_count={}",
        virtual_ts.len(),
        source_mappings.len()
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
            log_preview(&diag.message, 100)
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
                    log_preview(&diag.message, 80)
                );
                return None;
            }

            // Only an explicit source range owns an authored diagnostic.
            // Inference/navigation probes intentionally have no diagnostic
            // mapping; guessing a nearby line makes their errors duplicates.
            let (start_line, end_line, start_char, end_char) = map_diagnostic_with_source_mappings(
                virtual_ts,
                content,
                source_mappings,
                &virtual_result.import_source_map,
                diag.range.start.line,
                diag.range.start.character,
                diag.range.end.line,
                diag.range.end.character,
            )?;

            if is_authored_vue_import_extension_diagnostic(
                content, &diag, start_line, start_char, end_line, end_char,
            ) {
                tracing::debug!("skipping mapped .vue import extension diagnostic");
                return None;
            }

            let code = diag.code.as_ref().and_then(|code| match code {
                serde_json::Value::Number(number) => {
                    number.as_u64().and_then(|n| n.try_into().ok())
                }
                serde_json::Value::String(text) => text.trim_start_matches("TS").parse().ok(),
                _ => None,
            });
            let unreachable = LineBreaks::Lsp
                .position_to_offset(
                    virtual_ts,
                    diag.range.start.line,
                    diag.range.start.character,
                )
                .is_some_and(|offset| {
                    vize_canon::virtual_ts::is_unreachable_pattern_diagnostic(
                        virtual_ts, offset, code,
                    )
                });
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
                severity: if unreachable {
                    Some(DiagnosticSeverity::WARNING)
                } else {
                    diag.severity.map(|s| match s {
                        1 => DiagnosticSeverity::ERROR,
                        2 => DiagnosticSeverity::WARNING,
                        3 => DiagnosticSeverity::INFORMATION,
                        _ => DiagnosticSeverity::HINT,
                    })
                },
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

    let Some(start) = LineBreaks::Lsp.position_to_offset(
        virtual_ts,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    ) else {
        return false;
    };
    let Some(end) = LineBreaks::Lsp.position_to_offset(
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
        .is_some_and(authored_range_is_vue_sfc_specifier)
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

fn authored_range_is_vue_sfc_specifier(range: &str) -> bool {
    let specifier = range.trim_matches(|character| matches!(character, '\'' | '"' | '`'));
    let path = specifier
        .split_once(['?', '#'])
        .map_or(specifier, |(path, _)| path);
    path.ends_with(".vue")
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
#[path = "collect_virtual_tests.rs"]
mod tests;
