//! An initial Vue pass captures projection ownership before yielding native admission.
#![expect(
    clippy::disallowed_types,
    reason = "initial passes retain the actual native owner"
)]

use std::{path::PathBuf, sync::Arc};

use tower_lsp::lsp_types::{Diagnostic, Url};
use vize_canon::corsa_bridge::{InitialApiDiagnostics, PendingInitialApiDiagnostics};
use vize_canon::{
    CorsaBridge, CorsaBridgeError, CorsaVueVirtualDocument, CorsaVueVirtualDocumentOptions,
};

use super::super::{
    DiagnosticService,
    native::{has_blocking_parser_error, merge_complete_native, native_script_syntax},
};
use super::{
    collect::{CorsaDiagnostics, classify},
    collect_virtual::{CorsaDocument, assemble_corsa_diagnostics, finished_from_lsp},
};
use crate::server::ServerState;

pub(crate) struct PreparedInitialApiDiagnostics {
    pub(crate) bridge: Arc<CorsaBridge>,
    pub(crate) revision: u64,
    request: Option<InitialApiDiagnostics>,
    // Retain the exact projection, materialized source facts, source catalog and
    // native root captured while the existing request scope owned the world.
    opened: CorsaVueVirtualDocument,
    content: String,
    sync: Vec<Diagnostic>,
    native_script_syntax: bool,
}

impl PreparedInitialApiDiagnostics {
    pub(crate) async fn read(&mut self) -> Option<PendingInitialApiDiagnostics> {
        let request = self.request.take()?;
        Some(self.bridge.read_initial_api_diagnostics(request).await)
    }

    pub(crate) fn into_bridge(self) -> Arc<CorsaBridge> {
        self.bridge
    }

    pub(crate) fn dependencies(&self) -> &[PathBuf] {
        &self.opened.resolved_dependencies
    }

    pub(crate) fn complete(
        &self,
        diagnostics: Vec<vize_canon::corsa_bridge::LspDiagnostic>,
    ) -> Vec<Diagnostic> {
        let document = CorsaDocument {
            code: String::from(self.opened.code.as_str()),
            mapping: self.opened.mapping.clone(),
            import_source_map: self.opened.import_source_map.clone(),
        };
        let finished = diagnostics
            .into_iter()
            .filter_map(|diagnostic| finished_from_lsp(&document.code, diagnostic, 0))
            .collect();
        let native = assemble_corsa_diagnostics(&self.content, &[document], finished);
        merge_complete_native(self.sync.clone(), native, self.native_script_syntax)
    }
}

impl DiagnosticService {
    pub(crate) async fn prepare_initial_api_diagnostics(
        state: &ServerState,
        uri: &Url,
        sync: Vec<Diagnostic>,
    ) -> Result<Option<PreparedInitialApiDiagnostics>, (Arc<CorsaBridge>, CorsaBridgeError)> {
        // Art variants, scripts, JSX, disabled typecheck and parser fallback
        // retain their existing collector. This route never invents an LSP read.
        if !state.is_lsp_typecheck_enabled()
            || !state.lsp_features().has_diagnostics()
            || !uri.path().ends_with(".vue")
            || uri.path().ends_with(".art.vue")
        {
            return Ok(None);
        }
        let Some((revision, content)) = state
            .documents
            .get(uri)
            .map(|document| (document.revision(), document.text()))
        else {
            return Ok(None);
        };
        let native_script_syntax = native_script_syntax(state, uri, &sync);
        if content.contains("<art") || has_blocking_parser_error(&sync, native_script_syntax) {
            return Ok(None);
        }
        let Ok(source_path) = uri.to_file_path() else {
            return Ok(None);
        };
        let Some(bridge) = state.get_corsa_bridge().await else {
            return Ok(None);
        };
        let virtual_options = state.editor_virtual_ts_options().await;
        let cached_overlays = state.corsa_overlays();
        let overlays = cached_overlays
            .iter()
            .map(|(path, text)| (path.clone(), &**text))
            .collect::<Vec<_>>();
        let options = CorsaVueVirtualDocumentOptions {
            options_api: state.options_api_enabled(),
            legacy_vue2: state.legacy_vue2_enabled(),
            jsx_typecheck: state.jsx_typecheck_enabled(),
            experimental_patterned_template: state.patterned_template_enabled(),
            preserve_event_navigation: true,
            dialect: state.type_checker_vue_version(),
        };
        let opened = bridge
            .open_vue_virtual_document_with_borrowed_overlays_and_options(
                &source_path,
                &content,
                options,
                &overlays,
                &virtual_options,
            )
            .await
            .map_err(|error| (Arc::clone(&bridge), error))?;
        #[cfg(test)]
        state.record_editor_project_open("diagnostics", uri, &content, &opened);
        if std::env::var_os("VIZE_TRACE_EDITOR_PREPARATION").as_deref()
            == Some(std::ffi::OsStr::new("1"))
        {
            tracing::info!(target: "vize_editor_preparation", phase = "initial_api_read",
                event = "host_request_binding", host_uri = %uri,
                request_uri = %opened.request_uri, revision);
        }
        let request = bridge
            .prepare_initial_api_diagnostics(&opened.request_uri)
            .await
            .map_err(|error| (Arc::clone(&bridge), error))?;
        Ok(request.map(|request| PreparedInitialApiDiagnostics {
            bridge,
            revision,
            request: Some(request),
            opened,
            content,
            sync,
            native_script_syntax,
        }))
    }

    /// Called only after the owner has reacquired admission and rejected stale
    /// source/config/provider/session receipts. Preserve the original bounded
    /// classifier and one-retry contract for a completed current-world failure.
    pub(crate) async fn recover_initial_api_diagnostics(
        state: &ServerState,
        uri: &Url,
        sync: Vec<Diagnostic>,
        bridge: Arc<CorsaBridge>,
        error: CorsaBridgeError,
    ) -> Vec<Diagnostic> {
        let native_script_syntax = native_script_syntax(state, uri, &sync);
        // A timed-out read still owns its process and lease. Disclose missing
        // diagnostics without spawning a competing process while it drains.
        let result = if matches!(error, CorsaBridgeError::Timeout) && bridge.is_draining() {
            CorsaDiagnostics::Unavailable(vec![super::collect::typecheck_timed_out_hint()])
        } else {
            let failure = classify(&bridge, error);
            // Only the staged failure retains this owner now. Its original
            // attempt-0 arm drops/reaps it before retry opens a new process.
            drop(bridge);
            Self::collect_corsa_diagnostics_with_first_failure(state, uri, Some(failure)).await
        };
        let mut diagnostics = match result {
            CorsaDiagnostics::Complete(native) => {
                merge_complete_native(sync, native, native_script_syntax)
            }
            CorsaDiagnostics::Unavailable(hints) => {
                let mut diagnostics = sync;
                diagnostics.extend(hints);
                diagnostics
            }
        };
        // Match collect_async_from_sync's existing unavailable diagnosis.
        if !state.has_corsa_bridge()
            && !diagnostics.iter().any(|diagnostic| {
                diagnostic.source.as_deref() == Some(super::super::sources::TYPE_CHECKER)
            })
        {
            diagnostics.push(super::super::service::typecheck_unavailable_hint());
        }
        diagnostics
    }
}
