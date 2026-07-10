//! Orchestration of Corsa diagnostic collection for a single SFC document.

use std::path::Path;

use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::server::ServerState;

use super::super::DiagnosticService;
use super::collect_virtual::{
    collect_synced_virtual_result_diagnostics, collect_virtual_result_diagnostics,
};
use vize_canon::CorsaVueVirtualDocumentOptions;
use vize_carton::cstr;

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
        let mut diagnostics = if is_art_file {
            let Some(art_virtual) =
                Self::generate_virtual_ts_for_art_with_dependencies(uri, &content)
            else {
                tracing::warn!("failed to generate virtual ts for {}", uri);
                return vec![];
            };
            sync_art_vue_dependencies(
                &bridge,
                &art_virtual.vue_dependencies,
                CorsaVueVirtualDocumentOptions {
                    options_api,
                    legacy_vue2,
                },
            )
            .await;
            collect_virtual_result_diagnostics(
                &bridge,
                uri,
                content.as_str(),
                cstr!("{}.ts", uri.path()).to_string(),
                art_virtual.virtual_result,
            )
            .await
        } else {
            let Ok(source_path) = uri.to_file_path() else {
                tracing::warn!("cannot derive source path for {}", uri);
                return vec![];
            };
            let opened = match bridge
                .open_vue_virtual_document(
                    &source_path,
                    &content,
                    CorsaVueVirtualDocumentOptions {
                        options_api,
                        legacy_vue2,
                    },
                )
                .await
            {
                Ok(opened) => opened,
                Err(err) => {
                    tracing::warn!("failed to open Vue virtual document for {uri}: {err}");
                    return vec![];
                }
            };
            let Some((virtual_uri, virtual_result)) =
                Self::virtual_ts_result_from_corsa_vue_document(uri, &content, opened)
            else {
                tracing::warn!("failed to map virtual ts metadata for {}", uri);
                return vec![];
            };
            collect_synced_virtual_result_diagnostics(
                &bridge,
                uri,
                content.as_str(),
                virtual_uri,
                virtual_result,
            )
            .await
        };

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
                    )
                    .await,
                );
            }
        }

        diagnostics
    }
}

async fn sync_art_vue_dependencies(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    dependencies: &[std::path::PathBuf],
    options: CorsaVueVirtualDocumentOptions,
) {
    for dependency in dependencies {
        let Ok(content) = std::fs::read_to_string(dependency) else {
            continue;
        };
        if bridge
            .open_vue_virtual_document(dependency, &content, options)
            .await
            .is_ok()
        {
            continue;
        }

        let Some(fallback_uri) = vue_virtual_uri(dependency) else {
            continue;
        };
        if let Err(error) = bridge
            .open_or_update_virtual_document(
                fallback_uri.as_str(),
                "const component: any = undefined;\nexport default component;\n",
            )
            .await
        {
            tracing::debug!(
                "failed to sync Art Vue dependency fallback {}: {}",
                fallback_uri,
                error
            );
        }
    }
}

fn vue_virtual_uri(source_path: &Path) -> Option<String> {
    let virtual_path =
        source_path.with_file_name(cstr!("{}.ts", source_path.file_name()?.to_string_lossy()));
    Url::from_file_path(virtual_path).ok().map(Into::into)
}
