//! Orchestration of Corsa diagnostic collection for a single SFC document.

use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::server::ServerState;

use super::super::DiagnosticService;
use super::collect_virtual::collect_virtual_result_diagnostics;
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
pub(super) async fn overlay_sibling_vue_mirrors(
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
        let mut diagnostics = collect_virtual_result_diagnostics(
            &bridge,
            uri,
            content.as_str(),
            cstr!("{}.ts", uri.path()).to_string(),
            virtual_result,
            options_api,
            legacy_vue2,
        )
        .await;

        if !is_art_file {
            for (variant_index, inline_virtual) in Self::generate_virtual_ts_for_inline_art_variants(
                uri,
                &content,
                options_api,
                legacy_vue2,
            ) {
                diagnostics.extend(
                    collect_virtual_result_diagnostics(
                        &bridge,
                        uri,
                        content.as_str(),
                        cstr!("{}.inline_art_{variant_index}.ts", uri.path()).to_string(),
                        inline_virtual,
                        options_api,
                        legacy_vue2,
                    )
                    .await,
                );
            }
        }

        diagnostics
    }
}
