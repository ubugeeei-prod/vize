//! A first owned Vue request may initialize directly on its exact Canon graph.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use tower_lsp::lsp_types::Url;
use vize_canon::{
    CorsaBridge, CorsaBridgeError, CorsaInitialVueDocument, CorsaVueVirtualDocumentOptions,
    CorsaVueWorkspaceRequest,
};

use super::ServerState;

type StartupResult = (
    Result<(), CorsaBridgeError>,
    Option<CorsaInitialVueDocument>,
);

/// Borrowed on the warm path: no URI/source/owner allocation is needed there.
pub(super) struct InitialCanonicalSource<'a> {
    owner: &'a Arc<ServerState>,
    uri: &'a Url,
    source: &'a Arc<String>,
}

impl InitialCanonicalSource<'_> {
    fn is_current(&self, state: &ServerState, generation: u64) -> bool {
        std::ptr::eq(Arc::as_ptr(self.owner), state)
            && environment_is_current(state, generation)
            && state
                .documents
                .text_snapshot(self.uri)
                .is_some_and(|current| Arc::ptr_eq(&current, self.source))
    }
}

fn environment_is_current(state: &ServerState, generation: u64) -> bool {
    !state.project_context_retired()
        && state.corsa_environment_changes.load(Ordering::Acquire) == 0
        && state.corsa_environment_revision.load(Ordering::Acquire) == generation
}

impl ServerState {
    /// The ordinary strict open and the initial graph use one option projection.
    pub(crate) fn canonical_vue_document_options(&self) -> CorsaVueVirtualDocumentOptions {
        CorsaVueVirtualDocumentOptions {
            options_api: self.options_api_enabled(),
            legacy_vue2: self.legacy_vue2_enabled(),
            jsx_typecheck: self.jsx_typecheck_enabled(),
            experimental_patterned_template: self.patterned_template_enabled(),
            preserve_event_navigation: true,
            dialect: self.type_checker_vue_version(),
        }
    }

    /// Selected owned Vue callers retain a receipt only for their own request.
    /// All existing eager/direct callers keep get_corsa_bridge unchanged.
    pub(crate) async fn get_corsa_bridge_for_canonical_source(
        self: &Arc<Self>,
        uri: &Url,
        source: &Arc<String>,
    ) -> (Option<Arc<CorsaBridge>>, Option<CorsaInitialVueDocument>) {
        let initial = InitialCanonicalSource {
            owner: self,
            uri,
            source,
        };
        match self
            .get_corsa_bridge_with_initial_source(Some(initial))
            .await
        {
            Some((bridge, receipt)) => (Some(bridge), receipt),
            None => (None, None),
        }
    }

    pub(super) async fn spawn_initial_corsa_bridge(
        &self,
        bridge: &Arc<CorsaBridge>,
        initial: Option<&InitialCanonicalSource<'_>>,
        generation: u64,
    ) -> Option<StartupResult> {
        let Some(initial) = initial else {
            return Some((bridge.spawn().await, None));
        };
        if !initial.uri.path().ends_with(".vue") || initial.uri.path().ends_with(".art.vue") {
            return Some((bridge.spawn().await, None));
        }
        if !initial.is_current(self, generation) {
            return None;
        }
        let Some(source_path) = initial.uri.to_file_path().ok() else {
            return Some((bridge.spawn().await, None));
        };
        let authored = self.get_workspace_root();
        if !authored.as_ref().is_some_and(|root| {
            root.join("tsconfig.json").is_file() || root.join("jsconfig.json").is_file()
        }) {
            return Some((bridge.spawn().await, None));
        }

        // Missing providers keep the old fast failure/latch path without
        // materializing a complete graph or retaining an unknown-live mirror.
        if !bridge.can_resolve_initial_editor_runtime() {
            return Some((bridge.spawn().await, None));
        }
        let stamp = self.corsa_request_stamp();
        let virtual_ts_options = self.editor_virtual_ts_options().await;
        let overlays = self.corsa_overlays();
        if !initial.is_current(self, generation) || !stamp.is_current(self) {
            return None;
        }
        let request = CorsaVueWorkspaceRequest {
            source_path,
            content: initial.source.as_str().into(),
            options: self.canonical_vue_document_options(),
            overlays: overlays
                .iter()
                .map(|(path, content)| (path.clone(), Arc::clone(content)))
                .collect(),
            virtual_ts_options,
            requested_sources: Vec::new(),
        };
        let seed = bridge.prepare_initial_vue_project(request);
        if !initial.is_current(self, generation) || !stamp.is_current(self) {
            // A prepared seed owns rollback; no native job is submitted here.
            drop(seed);
            return None;
        }
        match seed {
            Ok(Some(seed)) => {
                let owner = Arc::clone(initial.owner);
                let result = bridge
                    .spawn_with_initial_project(seed, move || {
                        // Edits are checked by the semantic request's document stamp;
                        // they must not restart an otherwise current native owner.
                        environment_is_current(&owner, generation)
                    })
                    .await;
                Some(match result {
                    Ok(receipt) => (Ok(()), Some(receipt)),
                    Err(error) => (Err(error), None),
                })
            }
            Ok(None) => Some((bridge.spawn().await, None)),
            Err(error) => {
                // Graph/admission failure rolls back only the unstarted mirror.
                // The old eager and semantic error route must remain available.
                tracing::debug!(%error, "initial Canon graph uses ordinary eager startup");
                Some((bridge.spawn().await, None))
            }
        }
    }
}

#[cfg(test)]
mod tests;
