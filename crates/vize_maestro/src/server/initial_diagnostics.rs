//! Background scheduling for native type-diagnostic passes after open/edit.
//!
//! `tower-lsp` polls handlers concurrently, but Corsa's synchronous IPC can
//! occupy Maestro's single transport executor until a request completes. A
//! full type-diagnostic pass inside `didOpen` or `didChange` therefore makes an
//! interactive request wait behind work that is not required to answer it.
//! This module keeps native validation on one bounded background lane and
//! parser/lint feedback on a separate fixed lane. Both coalesce repeated edits
//! to the newest document version.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tower_lsp::lsp_types::Url;

use super::MaestroServer;

mod api_read;
mod importers;
use importers::refresh_initial_importers;
mod pending;
use super::state::{CorsaRequestStamp, ServerState};
use pending::{DiagnosticsJobCause, PendingInitialDiagnostics};
mod preparation;
mod retry;
mod worker;
pub(super) use retry::RetainedDiagnostics;
use worker::run_worker;

/// Let the editor's first interactive request claim Corsa before validation.
/// Initial diagnostics remain prompt, while completion/hover no longer queue
/// behind bridge startup and a full project diagnostic request.
const INTERACTIVE_GRACE: Duration = Duration::from_secs(1);
/// A short quiet period keeps an edit burst from queueing one whole native
/// diagnostics pass per document version. Interactive requests observe the
/// new buffer immediately and can claim Corsa while the user is still typing.
/// An edit skips it when nothing is queued for its document and the last pass
/// finished within it: that pass cannot hold Corsa longer than waiting would.
const EDIT_DIAGNOSTIC_DEBOUNCE: Duration = Duration::from_millis(150);

#[derive(Clone)]
pub(super) struct InitialDiagnosticsScheduler {
    sender: Option<mpsc::SyncSender<()>>,
    sync_sender: Option<mpsc::SyncSender<()>>,
    /// The active worker inserts into its queue without retaining a sender.
    worker_owned: bool,
    #[expect(
        clippy::disallowed_types,
        reason = "shared only with the single diagnostics worker"
    )]
    pending: std::sync::Arc<Mutex<PendingInitialDiagnostics>>,
}

impl InitialDiagnosticsScheduler {
    #[expect(
        clippy::disallowed_types,
        reason = "one bounded queue is shared with its worker thread"
    )]
    pub(super) fn new(mut worker: MaestroServer) -> Self {
        // The channel carries only a wake token. The authoritative queue keeps
        // at most one pending version per URI, so repeated opens cannot build
        // an unbounded FIFO backlog while Corsa is processing another file.
        let (sender, receiver) = mpsc::sync_channel(1);
        let (sync_sender, sync_receiver) = mpsc::sync_channel(1);
        let pending = std::sync::Arc::new(Mutex::new(PendingInitialDiagnostics::default()));
        let worker_pending = std::sync::Arc::clone(&pending);
        // Importer refreshes can finish a queued document's initial pass. Give
        // the worker completion access without retaining a channel sender,
        // which would prevent shutdown when the foreground server is dropped.
        worker.initial_diagnostics = Some(Self {
            sender: None,
            sync_sender: None,
            worker_owned: true,
            pending: std::sync::Arc::clone(&pending),
        });
        let mut sync_worker = MaestroServer::diagnostic_worker(
            worker.client.clone(),
            std::sync::Arc::clone(&worker.state),
        );
        sync_worker.initial_diagnostics = worker.initial_diagnostics.clone();
        let sync_pending = std::sync::Arc::clone(&pending);
        let native_wake = sender.clone();
        let sync_spawned = thread::Builder::new()
            .name("vize-sync-diagnostics".into())
            .spawn(move || {
                run_sync_worker(&sync_worker, sync_receiver, &sync_pending, native_wake)
            });
        if let Err(error) = sync_spawned {
            tracing::error!("failed to start sync diagnostics worker: {error}");
            return Self {
                sender: None,
                sync_sender: None,
                worker_owned: false,
                pending,
            };
        }
        let spawned = thread::Builder::new()
            .name("vize-initial-diagnostics".into())
            .spawn(move || run_worker(&worker, receiver, &worker_pending));

        match spawned {
            Ok(_) => Self {
                sender: Some(sender),
                sync_sender: Some(sync_sender),
                worker_owned: false,
                pending,
            },
            Err(error) => {
                tracing::error!("failed to start initial diagnostics worker: {error}");
                Self {
                    sender: None,
                    sync_sender: None,
                    worker_owned: false,
                    pending,
                }
            }
        }
    }

    fn schedule(
        &self,
        uri: Url,
        version: i32,
        cause: DiagnosticsJobCause,
        sync: Option<super::diagnostic_publishing::SyncDiagnostics>,
        continuation: Option<super::state::InitialConfigurationContinuation>,
        documents: &crate::document::DocumentStore,
    ) -> bool {
        let Some(sender) = &self.sender else {
            return false;
        };
        {
            let mut pending = self.pending.lock();
            if documents.version(&uri) != Some(version) {
                // Superseded or closed work needs no stale inline fallback.
                return true;
            }
            // Versions may restart after a close/reopen. The store, rather
            // than an older queued lifetime, owns the current version.
            if pending
                .jobs
                .get(&uri)
                .is_some_and(|job| job.version > version)
            {
                pending.jobs.remove(&uri);
            }
            let now = Instant::now();
            let not_before = match cause {
                DiagnosticsJobCause::Open => now + INTERACTIVE_GRACE,
                DiagnosticsJobCause::Edit => pending.edit_not_before(&uri, now),
            };
            pending.insert_with_continuation(uri, version, not_before, cause, sync, continuation);
        }
        if let Some(sync_sender) = &self.sync_sender {
            let _ = sync_sender.try_send(());
        }
        match sender.try_send(()) {
            Ok(()) | Err(mpsc::TrySendError::Full(())) => true,
            Err(mpsc::TrySendError::Disconnected(())) => {
                tracing::warn!("initial diagnostics worker stopped before accepting a document");
                self.pending.lock().jobs.clear();
                false
            }
        }
    }

    pub(super) fn complete_if_current(
        &self,
        uri: &Url,
        version: i32,
        stamp: CorsaRequestStamp,
        state: &ServerState,
    ) {
        let mut pending = self.pending.lock();
        // Only atomic revision reads occur under this lock. A new open cannot
        // enqueue until it is released, so an old pass cannot consume its job.
        if stamp.is_current(state) {
            pending.complete(uri, version);
        }
    }

    pub(super) fn discard_closed(&self, uri: &Url, documents: &crate::document::DocumentStore) {
        self.pending.lock().discard_closed(uri, documents);
    }

    pub(super) fn retry(
        &self,
        uri: Url,
        version: i32,
        documents: &crate::document::DocumentStore,
    ) -> bool {
        if self.sender.is_none() && !self.worker_owned {
            return false;
        }
        {
            let mut pending = self.pending.lock();
            if documents.version(&uri) != Some(version) {
                return false;
            }
            if pending
                .jobs
                .get(&uri)
                .is_some_and(|job| job.version > version)
            {
                pending.jobs.remove(&uri);
            }
            if !pending
                .jobs
                .get(&uri)
                .is_some_and(|job| job.version >= version)
            {
                pending.insert(uri.clone(), version, Instant::now() + INTERACTIVE_GRACE);
                if let Some(job) = pending.jobs.get_mut(&uri) {
                    // This retry needs a complete pass; its initial feedback
                    // was already attempted. Keep existing initial jobs intact.
                    job.sync_pending = false;
                }
            }
        }
        match self.sender.as_ref().map(|sender| sender.try_send(())) {
            None | Some(Ok(()) | Err(mpsc::TrySendError::Full(()))) => true,
            Some(Err(mpsc::TrySendError::Disconnected(()))) => {
                self.pending.lock().jobs.clear();
                false
            }
        }
    }
}

/// A fixed second lane keeps new-file parser/lint feedback independent of the
/// one shared native project warm-up. Both queues remain coalesced and bounded.
fn run_sync_worker(
    worker: &MaestroServer,
    receiver: mpsc::Receiver<()>,
    pending: &Mutex<PendingInitialDiagnostics>,
    native_wake: mpsc::SyncSender<()>,
) {
    while receiver.recv().is_ok() {
        loop {
            let sync_job = { pending.lock().take_sync() };
            let Some((uri, job)) = sync_job else {
                break;
            };
            crate::runtime::block_on(async {
                let phase = vize_canon::corsa_bridge::EditorPreparationPhase::start(
                    "initial_sync_configuration_admission",
                    1,
                );
                let project = worker
                    .for_initial_document(&uri, job.version, job.continuation.as_ref())
                    .await;
                phase.finish();
                project
                    .publish_initial_sync_diagnostics(&uri, job.version)
                    .await;
            });
            pending.lock().complete_sync(&uri, job.sequence);
            let _ = native_wake.try_send(());
        }
    }
}

impl MaestroServer {
    pub(super) fn schedule_initial_diagnostics(&self, uri: Url, version: i32) -> bool {
        self.initial_diagnostics.as_ref().is_some_and(|scheduler| {
            scheduler.schedule(
                uri.clone(),
                version,
                DiagnosticsJobCause::Open,
                None,
                self.state.initial_configuration_continuation(&uri, version),
                &self.state.documents,
            )
        })
    }

    pub(super) fn schedule_changed_diagnostics(
        &self,
        uri: Url,
        version: i32,
        sync: Option<super::diagnostic_publishing::SyncDiagnostics>,
    ) -> bool {
        self.initial_diagnostics.as_ref().is_some_and(|scheduler| {
            scheduler.schedule(
                uri.clone(),
                version,
                DiagnosticsJobCause::Edit,
                sync,
                None,
                &self.state.documents,
            )
        })
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod open_importer_tests;
