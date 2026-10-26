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
use vize_l0::FxHashMap;

use super::MaestroServer;

mod api_read;
mod preparation;
mod retry;
pub(super) use retry::RetainedDiagnostics;

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
const MAX_PENDING_DOCUMENTS: usize = 64;

#[derive(Clone)]
struct InitialDiagnosticsJob {
    cause: DiagnosticsJobCause,
    version: i32,
    not_before: Instant,
    sequence: u64,
    sync_pending: bool,
    sync_running: bool,
    preparation_pending: bool,
    sync: Option<super::diagnostic_publishing::SyncDiagnostics>,
    continuation: Option<super::state::InitialConfigurationContinuation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DiagnosticsJobCause {
    Open,
    Edit,
}

#[derive(Default)]
struct PendingInitialDiagnostics {
    jobs: FxHashMap<Url, InitialDiagnosticsJob>,
    next_sequence: u64,
    /// Lane occupancy of the last native pass; unknown until one finishes.
    last_pass: Option<Duration>,
}

impl PendingInitialDiagnostics {
    fn complete(&mut self, uri: &Url, version: i32) {
        if self.jobs.get(uri).is_some_and(|job| job.version <= version) {
            self.jobs.remove(uri);
        }
    }

    fn edit_not_before(&self, uri: &Url, now: Instant) -> Instant {
        let cheap = self
            .last_pass
            .is_some_and(|pass| pass < EDIT_DIAGNOSTIC_DEBOUNCE);
        if cheap && !self.jobs.contains_key(uri) {
            now
        } else {
            now + EDIT_DIAGNOSTIC_DEBOUNCE
        }
    }

    fn insert(&mut self, uri: Url, version: i32, not_before: Instant) {
        self.insert_with_continuation(
            uri,
            version,
            not_before,
            DiagnosticsJobCause::Open,
            None,
            None,
        );
    }

    fn insert_with_continuation(
        &mut self,
        uri: Url,
        version: i32,
        not_before: Instant,
        cause: DiagnosticsJobCause,
        sync: Option<super::diagnostic_publishing::SyncDiagnostics>,
        continuation: Option<super::state::InitialConfigurationContinuation>,
    ) {
        if self
            .jobs
            .get(&uri)
            .is_some_and(|pending| pending.version > version)
        {
            return;
        }

        if !self.jobs.contains_key(&uri) && self.jobs.len() == MAX_PENDING_DOCUMENTS {
            let oldest = self
                .jobs
                .iter()
                .min_by_key(|(_, job)| job.sequence)
                .map(|(uri, _)| uri.clone());
            if let Some(oldest) = oldest {
                self.jobs.remove(&oldest);
            }
        }

        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.jobs.insert(
            uri,
            InitialDiagnosticsJob {
                cause,
                version,
                not_before,
                sequence,
                sync_pending: cause == DiagnosticsJobCause::Open,
                sync_running: false,
                preparation_pending: cause == DiagnosticsJobCause::Open,
                sync,
                continuation,
            },
        );
    }

    fn take_sync(&mut self) -> Option<(Url, InitialDiagnosticsJob)> {
        let uri = self
            .jobs
            .iter()
            .filter(|(_, job)| job.sync_pending)
            .min_by_key(|(_, job)| job.sequence)
            .map(|(uri, _)| uri.clone())?;
        let job = self.jobs.get_mut(&uri)?;
        job.sync_pending = false;
        job.sync_running = true;
        Some((uri, job.clone()))
    }

    fn complete_sync(&mut self, uri: &Url, sequence: u64) {
        if let Some(job) = self
            .jobs
            .get_mut(uri)
            .filter(|job| job.sequence == sequence)
        {
            job.sync_running = false;
        }
    }

    fn take_preparation(&mut self) -> Option<(Url, InitialDiagnosticsJob)> {
        let uri = self
            .jobs
            .iter()
            .filter(|(_, job)| !job.sync_pending && !job.sync_running && job.preparation_pending)
            .min_by_key(|(_, job)| job.sequence)
            .map(|(uri, _)| uri.clone())?;
        let job = self.jobs.get_mut(&uri)?;
        job.preparation_pending = false;
        Some((uri, job.clone()))
    }

    fn next_not_before(&self) -> Option<Instant> {
        self.jobs
            .values()
            .filter(|job| !job.sync_pending && !job.sync_running)
            .map(|job| job.not_before)
            .min()
    }

    fn take_ready(&mut self, now: Instant) -> Option<(Url, InitialDiagnosticsJob)> {
        let uri = self
            .jobs
            .iter()
            .filter(|(_, job)| !job.sync_pending && !job.sync_running && job.not_before <= now)
            .min_by_key(|(_, job)| job.not_before)
            .map(|(uri, _)| uri.clone())?;
        self.jobs.remove(&uri).map(|job| (uri, job))
    }
}

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
    ) -> bool {
        let Some(sender) = &self.sender else {
            return false;
        };
        {
            let mut pending = self.pending.lock();
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

    pub(super) fn complete(&self, uri: &Url, version: i32) {
        self.pending.lock().complete(uri, version);
    }

    pub(super) fn retry(&self, uri: Url, version: i32) -> bool {
        if self.sender.is_none() && !self.worker_owned {
            return false;
        }
        {
            let mut pending = self.pending.lock();
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

fn run_worker(
    worker: &MaestroServer,
    receiver: mpsc::Receiver<()>,
    pending: &Mutex<PendingInitialDiagnostics>,
) {
    while receiver.recv().is_ok() {
        loop {
            let preparation = { pending.lock().take_preparation() };
            if let Some((uri, job)) = preparation {
                crate::runtime::block_on(async {
                    let project = worker
                        .for_initial_document(&uri, job.version, job.continuation.as_ref())
                        .await;
                    project
                        .prepare_initial_native_document(&uri, job.version)
                        .await;
                });
                continue;
            }
            let Some(not_before) = pending.lock().next_not_before() else {
                break;
            };
            let delay = not_before.saturating_duration_since(Instant::now());
            if !delay.is_zero() {
                match receiver.recv_timeout(delay) {
                    Ok(()) => continue,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
            let Some((uri, job)) = pending.lock().take_ready(Instant::now()) else {
                continue;
            };
            tracing::info!(
                "starting initial type diagnostics for {} version {}",
                uri,
                job.version
            );
            let started = Instant::now();
            crate::runtime::block_on(async {
                let phase = vize_canon::corsa_bridge::EditorPreparationPhase::start(
                    "initial_type_configuration_admission",
                    1,
                );
                let project = match job.cause {
                    DiagnosticsJobCause::Open => {
                        worker
                            .for_initial_document(&uri, job.version, job.continuation.as_ref())
                            .await
                    }
                    DiagnosticsJobCause::Edit => worker.for_document(&uri).await,
                };
                phase.finish();
                match job.cause {
                    DiagnosticsJobCause::Open => {
                        project
                            .publish_initial_api_diagnostics_if_version(
                                &uri,
                                job.version,
                                job.continuation.as_ref(),
                            )
                            .await;
                    }
                    DiagnosticsJobCause::Edit => {
                        project
                            .publish_changed_diagnostics_if_version(&uri, job.version, job.sync)
                            .await;
                    }
                }
                tracing::info!(
                    "finished initial type diagnostics for {} version {}",
                    uri,
                    job.version
                );
                // Opening an unsaved dependency changes its importers too.
                refresh_initial_importers(&project, &uri, job.version).await;
            });
            pending.lock().last_pass = Some(started.elapsed());
        }
    }
}

async fn refresh_initial_importers(project: &MaestroServer, uri: &Url, version: i32) -> Vec<Url> {
    // didOpen/didChange or the event that retained this pass prepared its
    // current owner already. Only other importer owners still need marking.
    project
        .publish_importer_diagnostics_with_current_owner(uri, Some(version), true)
        .await
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
            scheduler.schedule(uri.clone(), version, DiagnosticsJobCause::Edit, sync, None)
        })
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod open_importer_tests;
