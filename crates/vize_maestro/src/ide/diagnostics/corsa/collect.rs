//! Orchestration of Corsa diagnostic collection for a single SFC document.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

use crate::server::ServerState;

use super::super::{DiagnosticService, sources};
use super::mapping::{map_diagnostic_with_source_mappings, source_offset_to_position};
use super::message::rewrite_corsa_message;
use vize_carton::cstr;

/// Overlay the virtual TS for every relative `.vue` import of `host_uri`
/// into the editor's Corsa session so TypeScript module resolution can find
/// them (issue #752). Each sibling is opened at `<sibling_abs_path>.ts`,
/// matching the `.vue.ts` suffix produced by `ImportRewriter`. Transitive
/// `.vue` imports are followed recursively; cycles are avoided via a
/// visited set keyed on the canonicalized sibling path. Failures are
/// logged and skipped — a missing sibling falls through to the existing
/// TS2307 surface, which is the desired behavior for genuinely missing
/// modules.
async fn overlay_sibling_vue_mirrors(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    host_uri: &Url,
    initial_specifiers: &[std::string::String],
    options_api: bool,
    legacy_vue2: bool,
) {
    use std::collections::HashSet;
    use std::path::PathBuf;

    if initial_specifiers.is_empty() {
        return;
    }

    let Ok(host_path) = host_uri.to_file_path() else {
        tracing::debug!("overlay_sibling_vue_mirrors: host URI is not a file path: {host_uri}",);
        return;
    };
    let host_dir = match host_path.parent() {
        Some(dir) => dir.to_path_buf(),
        None => return,
    };

    let mut visited: HashSet<PathBuf> = HashSet::new();
    visited.insert(host_path.clone());

    let mut queue: Vec<(PathBuf, Vec<std::string::String>)> =
        vec![(host_dir, initial_specifiers.to_vec())];

    while let Some((dir, specifiers)) = queue.pop() {
        for specifier in specifiers {
            let resolved = dir.join(specifier.as_str());
            // Canonicalize is best-effort: if the file doesn't exist we still
            // try the lexical join so genuinely missing imports surface
            // TS2307 just as before.
            let canonical = std::fs::canonicalize(&resolved).unwrap_or(resolved);
            if !visited.insert(canonical.clone()) {
                continue;
            }

            let sibling_content = match std::fs::read_to_string(&canonical) {
                Ok(text) => text,
                Err(err) => {
                    tracing::debug!(
                        "overlay sibling skipped — read failed for {}: {err}",
                        canonical.display(),
                    );
                    continue;
                }
            };

            let sibling_uri = match Url::from_file_path(&canonical) {
                Ok(uri) => uri,
                Err(_) => continue,
            };

            let sibling_virtual = if canonical.to_string_lossy().ends_with(".art.vue") {
                DiagnosticService::generate_virtual_ts_for_art(&sibling_uri, &sibling_content)
            } else {
                DiagnosticService::generate_virtual_ts(
                    &sibling_uri,
                    &sibling_content,
                    options_api,
                    legacy_vue2,
                )
            };
            let Some(sibling_virtual) = sibling_virtual else {
                continue;
            };

            let sibling_name = cstr!("{}.ts", canonical.to_string_lossy());
            if let Err(err) = bridge
                .open_or_update_virtual_document(&sibling_name, &sibling_virtual.code)
                .await
            {
                tracing::debug!("overlay sibling failed for {}: {err}", canonical.display(),);
                continue;
            }

            let next_dir = canonical
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| canonical.clone());
            if !sibling_virtual.relative_vue_imports.is_empty() {
                queue.push((next_dir, sibling_virtual.relative_vue_imports));
            }
        }
    }
}

impl DiagnosticService {
    /// Collect diagnostics from the Corsa project-session backend.
    pub(in crate::ide::diagnostics) async fn collect_corsa_diagnostics(
        state: &ServerState,
        uri: &Url,
    ) -> Vec<Diagnostic> {
        tracing::info!("collect_corsa_diagnostics: {}", uri);

        // Only process .vue files
        if !uri.path().ends_with(".vue") {
            tracing::debug!("skipping non-vue file: {}", uri);
            return vec![];
        }

        // Get document content
        let Some(doc) = state.documents.get(uri) else {
            tracing::warn!("document not found: {}", uri);
            return vec![];
        };
        let content = doc.text();

        // Get the shared Corsa bridge.
        tracing::info!("getting corsa bridge...");
        let Some(bridge) = state.get_corsa_bridge().await else {
            tracing::warn!("corsa bridge not available");
            return vec![];
        };
        tracing::info!("corsa bridge acquired");

        // Generate virtual TypeScript
        let is_art_file = uri.path().ends_with(".art.vue");
        let options_api = state.options_api_enabled();
        let legacy_vue2 = state.legacy_vue2_enabled();
        let virtual_result = if is_art_file {
            Self::generate_virtual_ts_for_art(uri, &content)
        } else {
            Self::generate_virtual_ts(uri, &content, options_api, legacy_vue2)
        };
        let Some(virtual_result) = virtual_result else {
            tracing::warn!("failed to generate virtual ts for {}", uri);
            return vec![];
        };
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

        // Create the virtual document name used to derive a stable URI.
        let virtual_name = cstr!("{}.ts", uri.path());

        // Issue #752: Overlay sibling `.vue.ts` mirrors for every relative
        // `.vue` import so TypeScript's module resolution succeeds against
        // the temp-dir Corsa session. Without this, `import App from
        // './app.vue'` rewrites correctly to `./app.vue.ts` but still
        // reports TS2307 because the sibling is not present.
        overlay_sibling_vue_mirrors(
            &bridge,
            uri,
            &virtual_result.relative_vue_imports,
            options_api,
            legacy_vue2,
        )
        .await;

        // Open or update the document in Corsa (uses didChange if already open).
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

        // Get diagnostics (this polls until publishDiagnostics has landed).
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

        // Log each diagnostic for debugging
        for (i, diag) in corsa_diags.iter().enumerate() {
            tracing::info!(
                "  raw diag[{}]: line {}-{}, message: {}",
                i,
                diag.range.start.line,
                diag.range.end.line,
                &diag.message[..diag.message.len().min(100)]
            );
        }

        // Convert to LSP diagnostics with proper position mapping
        corsa_diags
            .into_iter()
            .filter_map(|diag| {
                // Skip warnings about internal generated variables
                // TS6133: 'X' is declared but its value is never read
                // TS6196: 'X' is declared but never used
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
                    content.as_str(),
                    source_mappings,
                    &virtual_result.import_source_map,
                    diag.range.start.line,
                    diag.range.start.character,
                    diag.range.end.line,
                    diag.range.end.character,
                );

                // Determine if this is a script error or template error.
                // Prefer byte-range source maps from canon. The older line
                // mapping remains as a fallback for diagnostics that land on
                // synthetic wrapper statements.
                let is_template_error = diag.range.start.line >= template_scope_start_line;

                let (start_line, end_line, start_char, end_char) = if let Some(mapped_range) = mapped_range {
                    mapped_range
                } else if is_template_error {
                    // Template scope error - try to find source mapping from @vize-map comments
                    let virtual_line = diag.range.start.line as usize;

                    // @vize-map comments are placed AFTER the code line they map.
                    // So for an error at line N, the mapping is at line N (from comment at N+1).
                    // Search forward (down) from the error line to find the mapping.
                    let mapping = (0..=10)
                        .filter_map(|offset| {
                            let search_line = virtual_line + offset;
                            line_mappings.get(search_line).and_then(|m| m.as_ref())
                        })
                        .next();

                    if let Some(src_mapping) = mapping {
                        // Found a source mapping - convert byte offset to line/column
                        let (start_line, start_col) =
                            source_offset_to_position(&content, src_mapping.start as usize);
                        let (end_line, end_col) =
                            source_offset_to_position(&content, src_mapping.end as usize);

                        tracing::info!(
                            "template error with mapping: virtual_line={} -> offset {}:{} -> sfc_line={} (message: {})",
                            diag.range.start.line,
                            src_mapping.start,
                            src_mapping.end,
                            start_line,
                            &diag.message[..diag.message.len().min(50)]
                        );
                        (start_line, end_line, start_col, end_col)
                    } else {
                        // No mapping found - skip this diagnostic
                        tracing::debug!(
                            "skipping unmapped template error at line {}: {}",
                            diag.range.start.line,
                            &diag.message[..diag.message.len().min(50)]
                        );
                        return None;
                    }
                } else {
                    // Skip diagnostics in generated preamble when no source map
                    // points back to user code.
                    if diag.range.start.line < user_code_start_line {
                        tracing::debug!(
                            "skipping preamble diagnostic at line {} (user code starts at {}): {}",
                            diag.range.start.line,
                            user_code_start_line,
                            &diag.message[..diag.message.len().min(50)]
                        );
                        return None;
                    }

                    // Script error - map using user code offset
                    let user_code_offset =
                        diag.range.start.line.saturating_sub(user_code_start_line);
                    let user_code_offset_end =
                        diag.range.end.line.saturating_sub(user_code_start_line);

                    // sfc_script_start_line is 1-indexed, convert to 0-indexed
                    // Add skipped_import_lines to account for import lines that were moved to module scope
                    let skipped_lines = virtual_result.skipped_import_lines;
                    let start =
                        (sfc_script_start_line.saturating_sub(1)) + user_code_offset + skipped_lines;
                    let end = (sfc_script_start_line.saturating_sub(1))
                        + user_code_offset_end
                        + skipped_lines;

                    // Adjust character offset: virtual TS adds 2 spaces of indentation
                    let start_ch = diag.range.start.character.saturating_sub(2);
                    let end_ch = diag.range.end.character.saturating_sub(2);

                    tracing::debug!(
                        "script error: virtual_line={} -> sfc_line={} (skipped_imports={}, message: {})",
                        diag.range.start.line,
                        start,
                        skipped_lines,
                        &diag.message[..diag.message.len().min(50)]
                    );
                    (start, end, start_ch, end_ch)
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
}
