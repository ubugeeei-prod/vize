//! Orchestration of Corsa diagnostic collection for a single SFC document.

use std::path::Path;
use std::sync::Arc;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range, Url};

use crate::server::ServerState;

use super::super::{DiagnosticService, sources};
use super::collect_virtual::{
    collect_synced_virtual_result_diagnostics, collect_virtual_result_diagnostics,
    deduplicate_diagnostics,
};
use vize_canon::{CorsaBridgeError, CorsaVueVirtualDocumentOptions};
use vize_s0::cstr;

/// One attempt either yields diagnostics (possibly empty for non-Corsa
/// reasons such as unsupported documents) or fails on a bridge call.
enum CollectFailure {
    /// The backend session answered but the request failed; not retried.
    Request(CorsaBridgeError),
    /// The backend process/transport is gone or unusable (crashed,
    /// OOM-killed, disconnected, or timed out). The carried bridge must be
    /// retired so a fresh session can be spawned (#3240, #3975).
    DeadBridge(Arc<vize_canon::CorsaBridge>, CorsaBridgeError),
}

fn classify(bridge: &Arc<vize_canon::CorsaBridge>, error: CorsaBridgeError) -> CollectFailure {
    match error {
        CorsaBridgeError::CommunicationError(_)
        | CorsaBridgeError::ProcessTerminated
        | CorsaBridgeError::Timeout => CollectFailure::DeadBridge(bridge.clone(), error),
        error => CollectFailure::Request(error),
    }
}

impl DiagnosticService {
    /// Collect diagnostics from the Corsa project-session backend.
    ///
    /// The backend is an external process that can die or stop answering
    /// mid-session. A failed bridge call therefore retires the unusable bridge
    /// and retries exactly once against a freshly spawned session, so the
    /// current publish can recover without another editor event. Repeated
    /// failures stay bounded: each collection performs at most one respawn,
    /// and spawn failures latch `corsa_init_failed`.
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
                    return match error {
                        CorsaBridgeError::Timeout => vec![typecheck_timed_out_hint()],
                        _ => vec![],
                    };
                }
                Err(CollectFailure::Request(error)) => {
                    tracing::warn!("corsa request failed for {uri}: {error}");
                    // A bound that fires has to be visible. Returning an empty
                    // list would make a timed-out pass indistinguishable from a
                    // clean file, which is the silence #3376 is about.
                    return match error {
                        CorsaBridgeError::Timeout => vec![typecheck_timed_out_hint()],
                        _ => vec![],
                    };
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

        // Own the document text up front: every `.await` below (bridge
        // acquisition, the background declaration scan, the bridge calls) can
        // hand the single executor thread to another queued handler, and a live
        // `documents.get` shard guard would deadlock the server against that
        // handler's `didOpen`/`didChange`/`didClose` write (#3315).
        let Some(content) = state.documents.text(uri) else {
            tracing::warn!("document not found: {}", uri);
            return Ok(vec![]);
        };

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
                    preserve_event_navigation: false,
                    dialect: state.type_checker_vue_version(),
                },
            )
            .await;
            let mut art_diagnostics = Vec::new();
            for variant in art_virtual.variants {
                art_diagnostics.extend(
                    collect_virtual_result_diagnostics(
                        &bridge,
                        uri,
                        content.as_str(),
                        super::virtual_ts_art::art_variant_virtual_name(uri, variant.variant_index),
                        variant.virtual_result,
                    )
                    .await
                    .map_err(|error| classify(&bridge, error))?,
                );
            }
            art_diagnostics
        } else {
            let Ok(source_path) = uri.to_file_path() else {
                tracing::warn!("cannot derive source path for {}", uri);
                return Ok(vec![]);
            };
            // Incrementally refreshed: documents unchanged since the last pass
            // keep their cached text instead of being copied out of their ropes
            // again, so a keystroke costs one document rather than every open
            // one (#3442). The snapshot is owned and lock-free, so holding it
            // across the bridge `.await` below is safe (#3315).
            let cached_overlays = state.corsa_overlays();
            let overlays = cached_overlays
                .iter()
                .map(|(path, text)| (path.clone(), &**text))
                .collect::<Vec<(std::path::PathBuf, &str)>>();
            let opened = bridge
                .open_vue_virtual_document_with_borrowed_overlays_and_options(
                    &source_path,
                    &content,
                    CorsaVueVirtualDocumentOptions {
                        options_api,
                        legacy_vue2,
                        preserve_event_navigation: false,
                        dialect: state.type_checker_vue_version(),
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

        // One authored problem inside the shared script context is reported by
        // every variant document that includes it, so the per-document dedup
        // has to be repeated across the whole set.
        Ok(deduplicate_diagnostics(diagnostics))
    }
}

/// Diagnostic published when a Corsa request outran the bridge's hard bound.
///
/// The bound is enforced by the bridge worker thread, because the request is
/// synchronous IPC and no async `timeout` around it can ever be polled
/// (#3376). This diagnostic is how the enforcement reaches the client — the
/// publish arriving promptly, and saying why the types are missing, is the
/// observable difference from a frozen server.
fn typecheck_timed_out_hint() -> Diagnostic {
    Diagnostic {
        range: Range::default(),
        severity: Some(DiagnosticSeverity::WARNING),
        code: Some(NumberOrString::String("typecheck-timed-out".to_string())),
        source: Some(sources::TYPE_CHECKER.to_string()),
        message: "Type checking timed out for this file: the Corsa runtime did not \
            answer within the request timeout, so type errors are missing from \
            these diagnostics."
            .to_string(),
        ..Default::default()
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use vize_canon::{CorsaBridge, CorsaBridgeError};

    use super::{CollectFailure, classify};

    #[test]
    fn a_timed_out_backend_is_retired_for_the_bounded_retry() {
        let bridge = Arc::new(CorsaBridge::new());

        match classify(&bridge, CorsaBridgeError::Timeout) {
            CollectFailure::DeadBridge(failed, CorsaBridgeError::Timeout) => {
                assert!(Arc::ptr_eq(&failed, &bridge));
            }
            _ => panic!("a timeout must replace the unusable backend"),
        }
    }

    #[test]
    fn an_answered_backend_error_does_not_replace_the_session() {
        let bridge = Arc::new(CorsaBridge::new());
        let error = CorsaBridgeError::ResponseError {
            code: -1,
            message: "bad request".into(),
        };

        assert!(matches!(
            classify(&bridge, error),
            CollectFailure::Request(CorsaBridgeError::ResponseError { .. })
        ));
    }
}
