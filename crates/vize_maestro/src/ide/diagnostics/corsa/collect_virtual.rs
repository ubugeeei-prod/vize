//! Mapping Corsa virtual-document diagnostics back to the host SFC.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

use super::super::{VirtualTsResult, sources};
use super::collect::overlay_sibling_vue_mirrors;
use super::mapping::{map_diagnostic_with_source_mappings, source_offset_to_position};
use super::message::rewrite_corsa_message;

pub(super) async fn collect_virtual_result_diagnostics(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    host_uri: &Url,
    content: &str,
    virtual_name: String,
    virtual_result: VirtualTsResult,
    options_api: bool,
    legacy_vue2: bool,
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

    overlay_sibling_vue_mirrors(
        bridge,
        host_uri,
        &virtual_result.relative_vue_imports,
        options_api,
        legacy_vue2,
    )
    .await;

    tracing::info!("opening/updating virtual document: {}", virtual_name);
    let virtual_uri = match bridge
        .open_or_update_virtual_document(&virtual_name, virtual_ts)
        .await
    {
        Ok(uri) => {
            tracing::info!("virtual document opened/updated successfully: {}", uri);
            uri
        }
        Err(e) => {
            tracing::warn!("failed to open/update virtual document: {}", e);
            return vec![];
        }
    };

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

    corsa_diags
        .into_iter()
        .filter_map(|diag| {
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
                source: Some(sources::TYPE_CHECKER.to_string()),
                message: rewrite_corsa_message(&diag.message),
                ..Default::default()
            })
        })
        .collect()
}
