//! Orchestration of Corsa diagnostic collection for a single SFC document.

use std::path::Path;
use std::sync::Arc;

use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::server::ServerState;

use super::super::DiagnosticService;
use super::collect_virtual::{
    collect_synced_virtual_result_diagnostics, collect_virtual_result_diagnostics,
};
use vize_canon::{CorsaBridgeError, CorsaVueVirtualDocumentOptions};
use vize_carton::cstr;

/// One attempt either yields diagnostics (possibly empty for non-Corsa
/// reasons such as unsupported documents) or fails on a bridge call.
enum CollectFailure {
    /// The backend session answered but the request failed; not retried.
    Request(CorsaBridgeError),
    /// The backend process/transport is gone (crashed, OOM-killed, or
    /// otherwise unreachable). The carried bridge must be retired so a
    /// fresh session can be spawned (#3240).
    DeadBridge(Arc<vize_canon::CorsaBridge>, CorsaBridgeError),
}

fn classify(bridge: &Arc<vize_canon::CorsaBridge>, error: CorsaBridgeError) -> CollectFailure {
    match error {
        CorsaBridgeError::CommunicationError(_) | CorsaBridgeError::ProcessTerminated => {
            CollectFailure::DeadBridge(bridge.clone(), error)
        }
        error => CollectFailure::Request(error),
    }
}

impl DiagnosticService {
    /// Collect diagnostics from the Corsa project-session backend.
    ///
    /// The backend is an external process that can die mid-session. A failed
    /// bridge call therefore retires the dead bridge and retries exactly once
    /// against a freshly spawned session, so a single `didChange` after a
    /// backend crash already republishes correct typecheck diagnostics.
    /// Repeated crashes stay bounded: each collection performs at most one
    /// respawn, and spawn failures latch `corsa_init_failed`.
    pub(in crate::ide::diagnostics) async fn collect_corsa_diagnostics(
        state: &ServerState,
        uri: &Url,
    ) -> Vec<Diagnostic> {
        for attempt in 0..2 {
            match Self::try_collect_corsa_diagnostics(state, uri).await {
                Ok(diagnostics) => return diagnostics,
                Err(CollectFailure::DeadBridge(bridge, error)) if attempt == 0 => {
                    tracing::warn!(
                        "corsa backend unreachable for {uri} ({error}); respawning and retrying"
                    );
                    state.retire_corsa_bridge(&bridge);
                }
                Err(CollectFailure::DeadBridge(_, error)) => {
                    tracing::warn!("corsa retry failed for {uri}: {error}");
                    return vec![];
                }
                Err(CollectFailure::Request(error)) => {
                    tracing::warn!("corsa request failed for {uri}: {error}");
                    return vec![];
                }
            }
        }
        vec![]
    }

    async fn try_collect_corsa_diagnostics(
        state: &ServerState,
        uri: &Url,
    ) -> Result<Vec<Diagnostic>, CollectFailure> {
        tracing::info!("collect_corsa_diagnostics: {}", uri);

        // Only process .vue files
        if !uri.path().ends_with(".vue") {
            tracing::debug!("skipping non-vue file: {}", uri);
            return Ok(vec![]);
        }

        // Get document content
        let Some(doc) = state.documents.get(uri) else {
            tracing::warn!("document not found: {}", uri);
            return Ok(vec![]);
        };
        let content = doc.text();

        // Get the shared Corsa bridge.
        tracing::info!("getting corsa bridge...");
        let Some(bridge) = state.get_corsa_bridge().await else {
            tracing::warn!("corsa bridge not available");
            return Ok(vec![]);
        };
        tracing::info!("corsa bridge acquired");

        // Generate virtual TypeScript
        let is_art_file = uri.path().ends_with(".art.vue");
        let options_api = state.options_api_enabled();
        let legacy_vue2 = state.legacy_vue2_enabled();
        let mut virtual_ts_options = state.virtual_ts_options();
        virtual_ts_options.reference_paths = state
            .global_component_reference_paths()
            .await
            .iter()
            .map(|path| path.to_string_lossy().as_ref().into())
            .collect();
        let mut diagnostics = if is_art_file {
            let Some(art_virtual) = Self::generate_virtual_ts_for_art_with_dependencies(
                uri,
                &content,
                &virtual_ts_options,
            ) else {
                tracing::warn!("failed to generate virtual ts for {}", uri);
                return Ok(vec![]);
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
            .map_err(|error| classify(&bridge, error))?
        } else {
            let Ok(source_path) = uri.to_file_path() else {
                tracing::warn!("cannot derive source path for {}", uri);
                return Ok(vec![]);
            };
            let overlays = state
                .documents
                .iter()
                .filter_map(|document| {
                    Some((
                        document.key().to_file_path().ok()?,
                        document.value().text().into(),
                    ))
                })
                .collect::<Vec<(std::path::PathBuf, vize_carton::String)>>();
            let opened = bridge
                .open_vue_virtual_document_with_overlays_and_options(
                    &source_path,
                    &content,
                    CorsaVueVirtualDocumentOptions {
                        options_api,
                        legacy_vue2,
                    },
                    &overlays,
                    &virtual_ts_options,
                )
                .await
                .map_err(|error| classify(&bridge, error))?;
            let Some((virtual_uri, virtual_result)) =
                Self::virtual_ts_result_from_corsa_vue_document(uri, &content, opened)
            else {
                tracing::warn!("failed to map virtual ts metadata for {}", uri);
                return Ok(vec![]);
            };
            collect_synced_virtual_result_diagnostics(
                &bridge,
                uri,
                content.as_str(),
                virtual_uri,
                virtual_result,
            )
            .await
            .map_err(|error| classify(&bridge, error))?
        };

        if !is_art_file {
            for (variant_index, inline_virtual) in Self::generate_virtual_ts_for_inline_art_variants(
                uri,
                &content,
                options_api,
                legacy_vue2,
                &virtual_ts_options,
            ) {
                diagnostics.extend(
                    collect_virtual_result_diagnostics(
                        &bridge,
                        uri,
                        content.as_str(),
                        cstr!("{}.inline_art_{variant_index}.ts", uri.path()).to_string(),
                        inline_virtual,
                    )
                    .await
                    .map_err(|error| classify(&bridge, error))?,
                );
            }
        }

        Ok(diagnostics)
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
