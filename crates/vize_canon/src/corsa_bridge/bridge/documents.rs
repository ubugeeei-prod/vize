//! Open-document lifecycle and diagnostic requests.
#![expect(
    clippy::disallowed_types,
    reason = "owned SDK and mirror leases cross the initial read worker"
)]

use std::sync::{Arc, atomic::Ordering};

use vize_carton::{String, cstr};

use super::{
    CorsaBridge, InitialApiDiagnostics, InitialApiDiagnosticsOutcome,
    InitialApiDiagnosticsProvenanceOutcome, PendingInitialApiDiagnostics,
};
use crate::corsa_bridge::types::{
    CorsaBridgeError, LspDiagnostic, PhasedLspDiagnostic, TypeCheckResult, VIRTUAL_URI_SCHEME,
};

impl CorsaBridge {
    pub async fn open_virtual_document(
        &self,
        name: &str,
        content: &str,
    ) -> Result<String, CorsaBridgeError> {
        let timer = self.profiler.timer("open_virtual_document");
        let uri = normalize_document_uri(name);
        let content = content.to_owned();
        let result_uri = uri.clone();
        let cache_len = self
            .with_client(move |client| {
                client
                    .did_open_fast(uri.as_str(), content.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        if let Some(timer) = timer {
            timer.record(&self.profiler);
        }
        Ok(result_uri)
    }

    pub async fn open_or_update_virtual_document(
        &self,
        name: &str,
        content: &str,
    ) -> Result<String, CorsaBridgeError> {
        self.open_virtual_document(name, content).await
    }

    /// Open related exact identities in one native request. References and
    /// rename use this for importer-scoped package shadows.
    pub async fn open_virtual_documents_batch(
        &self,
        documents: &[(String, String)],
    ) -> Result<(), CorsaBridgeError> {
        self.open_owned_virtual_documents_batch(documents.to_vec())
            .await
    }

    /// Transfer a prepared batch directly to the native worker.
    pub(in crate::corsa_bridge) async fn open_owned_virtual_documents_batch(
        &self,
        owned: Vec<(String, String)>,
    ) -> Result<(), CorsaBridgeError> {
        let cache_len = self
            .with_client(move |client| {
                let documents = owned
                    .iter()
                    .map(|(uri, content)| (uri.as_str(), content.as_str()))
                    .collect::<Vec<_>>();
                client
                    .did_open_batch_fast(&documents)
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        Ok(())
    }

    /// Queue exactly the old borrowed native inputs while sharing only cached
    /// URI/code storage; the worker owns the payload even if its caller drops.
    pub(in crate::corsa_bridge) async fn open_native_document_batch(
        &self,
        owned: super::super::vue_document::CorsaNativeDocumentBatch,
    ) -> Result<(), CorsaBridgeError> {
        let cache_len = self
            .with_client(move |client| {
                let documents = owned
                    .iter()
                    .map(|(uri, content)| (uri.as_str(), content.as_str()))
                    .collect::<Vec<_>>();
                client
                    .did_open_batch_fast(&documents)
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        Ok(())
    }

    pub async fn update_virtual_document(
        &self,
        uri: &str,
        content: &str,
        _version: i32,
    ) -> Result<(), CorsaBridgeError> {
        let timer = self.profiler.timer("update_virtual_document");
        let uri = uri.to_owned();
        let content = content.to_owned();
        let cache_len = self
            .with_client(move |client| {
                client
                    .did_change(uri.as_str(), content.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        if let Some(timer) = timer {
            timer.record(&self.profiler);
        }
        Ok(())
    }

    pub async fn close_virtual_document(&self, uri: &str) -> Result<(), CorsaBridgeError> {
        let uri = uri.to_owned();
        let cache_len = self
            .with_client(move |client| {
                client
                    .did_close(uri.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        Ok(())
    }

    /// Forget the on-disk project view cached for editor requests so the next
    /// request re-reads workspace files that changed outside the overlays.
    pub async fn invalidate_disk_project_state(&self) -> Result<(), CorsaBridgeError> {
        self.with_client(|client| {
            client
                .invalidate_disk_project_state()
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }

    /// Record an external disk project-shape change without touching the
    /// backend on the foreground LSP notification path.
    pub fn mark_disk_project_state_dirty(&self) {
        self.editor_session.invalidate_disk_inputs();
        self.disk_project_state_dirty.store(true, Ordering::SeqCst);
    }

    /// Flush a deferred disk-state invalidation just before a Corsa request.
    pub async fn flush_disk_project_state_if_dirty(&self) -> Result<(), CorsaBridgeError> {
        if !self.disk_project_state_dirty.swap(false, Ordering::SeqCst) {
            return Ok(());
        }
        let mut flush = PendingInvalidation {
            dirty: &self.disk_project_state_dirty,
            completed: false,
        };
        self.invalidate_disk_project_state().await?;
        flush.completed = true;
        Ok(())
    }

    /// Must be called after the current native transaction has opened all
    /// canonical sources. Captures without issuing a diagnostic query.
    pub async fn prepare_initial_api_diagnostics(
        &self,
        uri: &str,
    ) -> Result<Option<InitialApiDiagnostics>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let read = self
            .with_client(move |client| {
                client
                    .prepare_initial_diagnostics_read(&uri)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;
        Ok(read.map(|read| InitialApiDiagnostics {
            read: Arc::new(read),
            mirror: Arc::clone(&self.editor_session),
        }))
    }

    /// The caller executes this synchronous transaction while holding its real
    /// source guard, then checks its full owner receipt. The existing exclusive
    /// client lane can roll back only its own row if that postguard refuses it.
    /// No error is classified or session retired before authority is accepted.
    pub async fn finish_initial_api_diagnostics<F>(
        &self,
        pending: PendingInitialApiDiagnostics,
        with_current_source: F,
    ) -> Result<InitialApiDiagnosticsOutcome, CorsaBridgeError>
    where
        F: FnOnce(&mut dyn FnMut() -> bool) -> bool + Send + 'static,
    {
        Ok(
            match self
                .finish_initial_api_diagnostics_with_provenance(pending, with_current_source)
                .await?
            {
                InitialApiDiagnosticsProvenanceOutcome::Stale => {
                    InitialApiDiagnosticsOutcome::Stale
                }
                InitialApiDiagnosticsProvenanceOutcome::Unsupported => {
                    InitialApiDiagnosticsOutcome::Unsupported
                }
                InitialApiDiagnosticsProvenanceOutcome::Complete(rows) => {
                    InitialApiDiagnosticsOutcome::Complete(
                        rows.into_iter().map(|row| row.diagnostic).collect(),
                    )
                }
                InitialApiDiagnosticsProvenanceOutcome::Failed(error) => {
                    InitialApiDiagnosticsOutcome::Failed(error)
                }
            },
        )
    }

    /// Commit the same guarded initial read while retaining its compiler phases.
    pub async fn finish_initial_api_diagnostics_with_provenance<F>(
        &self,
        pending: PendingInitialApiDiagnostics,
        with_current_source: F,
    ) -> Result<InitialApiDiagnosticsProvenanceOutcome, CorsaBridgeError>
    where
        F: FnOnce(&mut dyn FnMut() -> bool) -> bool + Send + 'static,
    {
        let (outcome, cache_len) = self
            .with_client(move |client| {
                let uri = pending.read.uri().to_owned();
                // Decode the inert owned response before taking the caller's
                // host-document guard; the guarded section only swaps a row.
                let read = pending.read;
                let (mut diagnostics, staged) = match pending.result {
                    Ok(Some(diagnostics)) => {
                        match convert_bridge_diagnostics_with_provenance(&diagnostics) {
                            Ok(mapped) => (
                                Some(diagnostics),
                                InitialApiDiagnosticsProvenanceOutcome::Complete(mapped),
                            ),
                            Err(error) => {
                                (None, InitialApiDiagnosticsProvenanceOutcome::Failed(error))
                            }
                        }
                    }
                    Ok(None) => (None, InitialApiDiagnosticsProvenanceOutcome::Unsupported),
                    Err(error) => (None, InitialApiDiagnosticsProvenanceOutcome::Failed(error)),
                };
                let mut staged = Some(staged);
                let mut previous = None;
                let mut inserted = false;
                let mut outcome = InitialApiDiagnosticsProvenanceOutcome::Stale;
                let accepted = with_current_source(&mut || {
                    if !client.initial_diagnostics_read_is_current(&read) {
                        return false;
                    }
                    let Some(candidate) = staged.take() else {
                        return false;
                    };
                    if let Some(diagnostics) = diagnostics.take() {
                        previous =
                            client.replace_initial_diagnostics_cache(&uri, Some(diagnostics));
                        inserted = true;
                    }
                    outcome = candidate;
                    true
                });
                if !accepted {
                    if inserted {
                        client.replace_initial_diagnostics_cache(&uri, previous);
                    }
                    outcome = InitialApiDiagnosticsProvenanceOutcome::Stale;
                }
                Ok((outcome, client.diagnostics_cache_len()))
            })
            .await?;
        if matches!(outcome, InitialApiDiagnosticsProvenanceOutcome::Complete(_)) {
            self.cache_stats.set_entries(cache_len as u64);
            self.cache_stats.miss();
        }
        Ok(outcome)
    }
    pub async fn get_diagnostics(&self, uri: &str) -> Result<Vec<LspDiagnostic>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let (used_cache, cache_len, diagnostics) = self
            .with_client(move |client| {
                let fetch = client
                    .request_diagnostics_full(uri.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                let diagnostics = convert_bridge_diagnostics(&fetch.diagnostics)?;
                Ok((
                    fetch.used_cache,
                    client.diagnostics_cache_len(),
                    diagnostics,
                ))
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        if used_cache {
            self.cache_stats.hit();
        } else {
            self.cache_stats.miss();
        }
        Ok(diagnostics)
    }

    /// Fetch the existing diagnostic request result with producer-owned provenance.
    pub async fn get_diagnostics_with_provenance(
        &self,
        uri: &str,
    ) -> Result<Vec<PhasedLspDiagnostic>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let (used_cache, cache_len, diagnostics) = self
            .with_client(move |client| {
                let fetch = client
                    .request_diagnostics_full_with_provenance(uri.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                let diagnostics = convert_bridge_diagnostics_with_provenance(&fetch.diagnostics)?;
                Ok((
                    fetch.used_cache,
                    client.diagnostics_cache_len(),
                    diagnostics,
                ))
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        if used_cache {
            self.cache_stats.hit();
        } else {
            self.cache_stats.miss();
        }
        Ok(diagnostics)
    }

    pub async fn type_check(
        &self,
        name: &str,
        content: &str,
    ) -> Result<TypeCheckResult, CorsaBridgeError> {
        let timer = self.profiler.timer("type_check");
        let uri = self.open_virtual_document(name, content).await?;
        let diagnostics = self.get_diagnostics(&uri).await?;
        if let Some(timer) = timer {
            timer.record(&self.profiler);
        }
        Ok(TypeCheckResult {
            diagnostics,
            source_map: None,
        })
    }
}

// A queued invalidation can now be cancelled while the worker is busy. Restore
// its dirty bit on cancellation/failure, while preserving newer watcher marks.
struct PendingInvalidation<'a> {
    dirty: &'a std::sync::atomic::AtomicBool,
    completed: bool,
}

impl Drop for PendingInvalidation<'_> {
    fn drop(&mut self) {
        if !self.completed {
            self.dirty.store(true, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod invalidation_tests;

pub(crate) fn normalize_document_uri(name: &str) -> String {
    if name.starts_with("file://") {
        name.into()
    } else if name.starts_with('/') || std::path::Path::new(name).is_absolute() {
        crate::file_uri::path_to_file_uri(std::path::Path::new(name))
    } else {
        cstr!("{VIRTUAL_URI_SCHEME}://{name}")
    }
}

pub(super) fn convert_bridge_diagnostics<T: std::borrow::Borrow<lsp_types::Diagnostic>>(
    diagnostics: &[T],
) -> Result<Vec<LspDiagnostic>, CorsaBridgeError> {
    diagnostics
        .iter()
        .map(|row| convert_bridge_diagnostic(row.borrow()))
        .collect()
}

fn convert_bridge_diagnostic(
    diagnostic: &lsp_types::Diagnostic,
) -> Result<LspDiagnostic, CorsaBridgeError> {
    let value = serde_json::to_value(diagnostic).map_err(|error| {
        CorsaBridgeError::CommunicationError(cstr!("Failed to encode diagnostic: {error}"))
    })?;
    serde_json::from_value(value).map_err(|error| {
        CorsaBridgeError::CommunicationError(cstr!("Failed to parse diagnostic: {error}"))
    })
}

pub(super) fn convert_bridge_diagnostics_with_provenance(
    diagnostics: &[crate::lsp_client::PhasedDiagnostic],
) -> Result<Vec<PhasedLspDiagnostic>, CorsaBridgeError> {
    diagnostics
        .iter()
        .map(|row| {
            Ok(PhasedLspDiagnostic {
                diagnostic: convert_bridge_diagnostic(&row.diagnostic)?,
                phase: row.phase,
            })
        })
        .collect()
}
