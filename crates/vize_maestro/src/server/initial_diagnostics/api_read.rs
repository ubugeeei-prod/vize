//! The initial API read yields foreground admission and retains its exact world.
#![expect(
    clippy::disallowed_types,
    reason = "staged reads retain their actual admitted owner"
)]

use std::sync::Arc;

use tower_lsp::lsp_types::{Diagnostic, Url};
use vize_canon::corsa_bridge::{InitialApiDiagnosticsOutcome, PendingInitialApiDiagnostics};
use vize_canon::{CorsaBridge, CorsaBridgeError};

use super::{MaestroServer, RetainedDiagnostics};
use crate::{
    ide::{
        DiagnosticService,
        diagnostics::{PreparedInitialApiDiagnostics, with_native_diagnostic_timeout},
    },
    server::{
        diagnostic_publishing::CollectedDiagnostics,
        state::{
            CorsaRequestScope, CorsaRequestStamp, InitialConfigurationContinuation, ServerState,
        },
    },
};

/// Exactly the existing source stamp, authored identity and ORIGINAL didOpen
/// continuation. This receipt never adopts a world discovered after the read.
#[derive(Clone)]
struct InitialApiWorld {
    uri: Url,
    version: i32,
    revision: u64,
    stamp: CorsaRequestStamp,
    continuation: InitialConfigurationContinuation,
}

impl InitialApiWorld {
    fn is_current(&self, state: &Arc<ServerState>) -> bool {
        self.stamp.is_current(state)
            && self
                .continuation
                .owns_current_state(state, &self.uri, self.version)
            && state.documents.get(&self.uri).is_some_and(|document| {
                document.version == self.version && document.revision() == self.revision
            })
    }

    fn owns_bridge(&self, state: &Arc<ServerState>, bridge: &Arc<CorsaBridge>) -> bool {
        self.is_current(state) && state.owns_corsa_bridge(bridge)
    }
}

enum InitialRead {
    Prepared(Box<(PreparedInitialApiDiagnostics, PendingInitialApiDiagnostics)>),
    Failed(Arc<CorsaBridge>, CorsaBridgeError),
}

impl MaestroServer {
    /// Only this initial regular-Vue route releases native admission. Every
    /// foreground/edit/save/importer route keeps its existing transaction.
    pub(in crate::server) async fn publish_initial_api_diagnostics_if_version(
        &self,
        uri: &Url,
        expected: i32,
        continuation: Option<&InitialConfigurationContinuation>,
    ) {
        // Preserve the original empty-provider, disabled-typecheck and retired
        // entry paths before any detached preparation or native scope change.
        if !self.state.lsp_features().has_diagnostics()
            || !self.state.is_lsp_typecheck_enabled()
            || self.state.project_context_retired()
        {
            self.publish_diagnostics_if_version(uri, expected).await;
            return;
        }
        let Some(continuation) = continuation
            .cloned()
            .filter(|receipt| receipt.owns_current_state(&self.state, uri, expected))
        else {
            self.publish_diagnostics_if_version(uri, expected).await;
            return;
        };
        let retained = RetainedDiagnostics::new(self, uri);
        let document_lock = self.state.diagnostic_lock(uri);
        let document_guard = document_lock.lock().await;
        let scope = self.state.corsa_request_scope().await;
        let Some(revision) = self
            .state
            .documents
            .get(uri)
            .filter(|document| document.version == expected)
            .map(|document| document.revision())
        else {
            return;
        };
        let world = InitialApiWorld {
            uri: uri.clone(),
            version: expected,
            revision,
            stamp: scope.stamp(),
            continuation,
        };
        if !world.is_current(&self.state) {
            return;
        }
        let sync = DiagnosticService::collect(&self.state, uri);
        // Preserve the original outer deadline over ALL native collection:
        // preparation, open, read, physical guards, finish, recovery and mapping.
        // Detached work does not restart a timeout or move the measured bound.
        let diagnostics = match with_native_diagnostic_timeout(
            self.collect_initial_api_with_scope(&world, scope, &sync),
        )
        .await
        {
            Ok(Some(diagnostics)) => diagnostics,
            Ok(None) => return,
            Err(hints) => {
                // The original timeout packet discloses missing type results.
                // Cancellation retires only the waiter; entered IPC still owns
                // its real snapshot on the bounded read lane until it drains.
                let mut diagnostics = sync;
                diagnostics.extend(hints);
                diagnostics
            }
        };
        if !world.is_current(&self.state) {
            return;
        }
        let collected = CollectedDiagnostics {
            version: expected,
            diagnostics,
            stamp: Some(world.stamp),
        };
        drop(document_guard);
        // Keep the original stamped publication CAS, duplicate handling,
        // diagnostic/lint cache semantics and unavailable notice.
        self.publish_collected_diagnostics(uri, collected).await;
        retained.finish();
    }

    async fn collect_initial_api_with_scope(
        &self,
        world: &InitialApiWorld,
        scope: CorsaRequestScope<'_>,
        sync: &[Diagnostic],
    ) -> Option<Vec<Diagnostic>> {
        let read = match DiagnosticService::prepare_initial_api_diagnostics(
            &self.state,
            &world.uri,
            sync.to_vec(),
        )
        .await
        {
            Ok(Some(mut prepared)) => {
                trace_initial_api(world, "prepared", None);
                if !world.owns_bridge(&self.state, &prepared.bridge) {
                    return None;
                }
                // The mutable client worker finished capture above. Drop the
                // global native scope before entering the independent API RPC.
                drop(scope);
                let pending = prepared.read().await?;
                InitialRead::Prepared(Box::new((prepared, pending)))
            }
            Ok(None) => {
                // Unsupported/parser/art/script paths keep their existing
                // collector and scope, inside this SAME original outer deadline.
                if !world.is_current(&self.state) {
                    return None;
                }
                trace_initial_api(world, "legacy_fallback", Some("prepare_none"));
                let diagnostics = DiagnosticService::collect_async_from_sync(
                    &self.state,
                    &world.uri,
                    sync.to_vec(),
                )
                .await;
                return world.is_current(&self.state).then_some(diagnostics);
            }
            Err((bridge, error)) => {
                trace_initial_api(world, "prepare_failed", None);
                drop(scope);
                InitialRead::Failed(bridge, error)
            }
        };
        // Reuse ORIGINAL full record/source-role/provider/owner validation:
        // TWO additional fresh physical guard passes on the existing config
        // worker, without Vite evaluation or retargeting. Their cost stays in
        // the original outer deadline; no count/performance equivalence claimed.
        let current = self
            .state
            .continue_initial_configuration(
                world.continuation.clone(),
                world.uri.clone(),
                world.version,
            )
            .await;
        if !current
            .as_ref()
            .is_some_and(|owner| Arc::ptr_eq(owner, &self.state))
        {
            return None;
        }
        // Never queue this background commit ahead of an interactive request.
        // A busy owner retains the existing bounded retry instead of spinning.
        let _scope = self.state.try_corsa_request_scope()?;
        if !world.is_current(&self.state) {
            return None;
        }
        let diagnostics = match read {
            InitialRead::Prepared(prepared) => {
                let (prepared, pending) = *prepared;
                if !world.owns_bridge(&self.state, &prepared.bridge) {
                    return None;
                }
                let source_state = Arc::clone(&self.state);
                let source_world = world.clone();
                let native_owner = Arc::downgrade(&prepared.bridge);
                let outcome = prepared
                    .bridge
                    .finish_initial_api_diagnostics(pending, move |commit| {
                        let Some(bridge) = native_owner.upgrade() else {
                            return false;
                        };
                        if !source_world.owns_bridge(&source_state, &bridge) {
                            return false;
                        }
                        let accepted = {
                            // Hold the actual host shard through cache insertion.
                            // Do not recursively acquire it via continuation checks:
                            // a queued document writer must not deadlock this guard.
                            let Some(document) = source_state.documents.get(&source_world.uri)
                            else {
                                return false;
                            };
                            if document.revision() != source_world.revision
                                || document.version != source_world.version
                                || !source_world.stamp.is_current(&source_state)
                                || !source_state.owns_corsa_bridge(&bridge)
                            {
                                return false;
                            }
                            commit()
                                && source_world.stamp.is_current(&source_state)
                                && source_state.owns_corsa_bridge(&bridge)
                        };
                        // The host guard has dropped. Recheck full original owner,
                        // routing/compiler/source authority before accepting the row.
                        // Refusal restores only this URI's previous row while the
                        // same client worker still excludes every newer cache writer.
                        accepted && source_world.owns_bridge(&source_state, &bridge)
                    })
                    .await;
                // Only now may a completed CURRENT-world error reach recovery.
                // Its guarded cache transaction accepted only the original source world.
                if !world.owns_bridge(&self.state, &prepared.bridge) {
                    return None;
                }
                match outcome {
                    Ok(InitialApiDiagnosticsOutcome::Stale) => {
                        trace_initial_api(world, "stale", None);
                        return None;
                    }
                    Ok(InitialApiDiagnosticsOutcome::Complete(native)) => {
                        trace_initial_api(world, "complete", None);
                        let diagnostics = prepared.complete(native);
                        if prepared.revision != world.revision
                            || !world.owns_bridge(&self.state, &prepared.bridge)
                            || !self.state.record_initial_typecheck_dependencies(
                                &world.uri,
                                world.revision,
                                prepared.dependencies(),
                                world.stamp,
                            )
                        {
                            return None;
                        }
                        diagnostics
                    }
                    Ok(InitialApiDiagnosticsOutcome::Unsupported) => {
                        trace_initial_api(world, "legacy_fallback", Some("unsupported"));
                        // Preserve the runtime's existing unsupported policy.
                        DiagnosticService::collect_async_from_sync(
                            &self.state,
                            &world.uri,
                            sync.to_vec(),
                        )
                        .await
                    }
                    Ok(InitialApiDiagnosticsOutcome::Failed(error)) | Err(error) => {
                        trace_initial_api(world, "recovery", None);
                        DiagnosticService::recover_initial_api_diagnostics(
                            &self.state,
                            &world.uri,
                            sync.to_vec(),
                            prepared.into_bridge(),
                            error,
                        )
                        .await
                    }
                }
            }
            InitialRead::Failed(bridge, error) => {
                if !world.owns_bridge(&self.state, &bridge) {
                    return None;
                }
                DiagnosticService::recover_initial_api_diagnostics(
                    &self.state,
                    &world.uri,
                    sync.to_vec(),
                    bridge,
                    error,
                )
                .await
            }
        };
        world.is_current(&self.state).then_some(diagnostics)
    }
}

fn trace_initial_api(world: &InitialApiWorld, event: &str, reason: Option<&str>) {
    if std::env::var_os("VIZE_TRACE_EDITOR_PREPARATION").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        tracing::info!(target: "vize_editor_preparation", phase = "initial_api_read",
            event, reason, host_uri = %world.uri, version = world.version,
            revision = world.revision);
    }
}
