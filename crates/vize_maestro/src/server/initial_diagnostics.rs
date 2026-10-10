//! Background scheduling for the first native type-diagnostic pass.
//!
//! `tower-lsp` polls handlers concurrently, but Corsa's synchronous IPC can
//! occupy Maestro's single transport executor until a request completes. A
//! full type-diagnostic pass inside `didOpen` therefore makes the first
//! completion wait behind work that is not required to answer it. This module
//! keeps that validation work while moving it onto one bounded background lane.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tower_lsp::lsp_types::Url;
use vize_l0::FxHashMap;

use super::MaestroServer;

mod retry;
pub(super) use retry::RetainedDiagnostics;

/// Let the editor's first interactive request claim Corsa before validation.
/// Initial diagnostics remain prompt, while completion/hover no longer queue
/// behind bridge startup and a full project diagnostic request.
const INTERACTIVE_GRACE: Duration = Duration::from_secs(1);
const MAX_PENDING_DOCUMENTS: usize = 64;

struct InitialDiagnosticsJob {
    version: i32,
    not_before: Instant,
    sequence: u64,
    sync_pending: bool,
}

#[derive(Default)]
struct PendingInitialDiagnostics {
    jobs: FxHashMap<Url, InitialDiagnosticsJob>,
    next_sequence: u64,
}

impl PendingInitialDiagnostics {
    fn complete(&mut self, uri: &Url, version: i32) {
        if self.jobs.get(uri).is_some_and(|job| job.version <= version) {
            self.jobs.remove(uri);
        }
    }

    fn insert(&mut self, uri: Url, version: i32, not_before: Instant) {
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
                version,
                not_before,
                sequence,
                sync_pending: true,
            },
        );
    }

    fn take_sync(&mut self) -> Option<(Url, i32)> {
        let uri = self
            .jobs
            .iter()
            .filter(|(_, job)| job.sync_pending)
            .min_by_key(|(_, job)| job.sequence)
            .map(|(uri, _)| uri.clone())?;
        let job = self.jobs.get_mut(&uri)?;
        job.sync_pending = false;
        Some((uri, job.version))
    }

    fn next_not_before(&self) -> Option<Instant> {
        self.jobs.values().map(|job| job.not_before).min()
    }

    fn take_ready(&mut self, now: Instant) -> Option<(Url, InitialDiagnosticsJob)> {
        let uri = self
            .jobs
            .iter()
            .filter(|(_, job)| job.not_before <= now)
            .min_by_key(|(_, job)| job.not_before)
            .map(|(uri, _)| uri.clone())?;
        self.jobs.remove(&uri).map(|job| (uri, job))
    }
}

#[derive(Clone)]
pub(super) struct InitialDiagnosticsScheduler {
    sender: Option<mpsc::SyncSender<()>>,
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
        let pending = std::sync::Arc::new(Mutex::new(PendingInitialDiagnostics::default()));
        let worker_pending = std::sync::Arc::clone(&pending);
        // Importer refreshes can finish a queued document's initial pass. Give
        // the worker completion access without retaining a channel sender,
        // which would prevent shutdown when the foreground server is dropped.
        worker.initial_diagnostics = Some(Self {
            sender: None,
            worker_owned: true,
            pending: std::sync::Arc::clone(&pending),
        });
        let spawned = thread::Builder::new()
            .name("vize-initial-diagnostics".into())
            .spawn(move || run_worker(&worker, receiver, &worker_pending));

        match spawned {
            Ok(_) => Self {
                sender: Some(sender),
                worker_owned: false,
                pending,
            },
            Err(error) => {
                tracing::error!("failed to start initial diagnostics worker: {error}");
                Self {
                    sender: None,
                    worker_owned: false,
                    pending,
                }
            }
        }
    }

    pub(super) fn schedule(&self, uri: Url, version: i32) -> bool {
        let Some(sender) = &self.sender else {
            return false;
        };
        self.pending
            .lock()
            .insert(uri, version, Instant::now() + INTERACTIVE_GRACE);
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

fn run_worker(
    worker: &MaestroServer,
    receiver: mpsc::Receiver<()>,
    pending: &Mutex<PendingInitialDiagnostics>,
) {
    while receiver.recv().is_ok() {
        loop {
            // Parsing and linting can be expensive on a cold SFC. Keep them
            // off the foreground didOpen handler while still publishing prompt
            // non-empty feedback before the delayed native type pass.
            let sync_job = { pending.lock().take_sync() };
            if let Some((uri, version)) = sync_job {
                crate::runtime::block_on(async {
                    let project = worker.for_document(&uri).await;
                    project
                        .publish_initial_sync_diagnostics(&uri, version)
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
            crate::runtime::block_on(async {
                let project = worker.for_document(&uri).await;
                project
                    .publish_diagnostics_if_version(&uri, job.version)
                    .await;
                tracing::info!(
                    "finished initial type diagnostics for {} version {}",
                    uri,
                    job.version
                );
                // Opening an unsaved dependency changes its importers too.
                project
                    .publish_importer_diagnostics(&uri, Some(job.version))
                    .await;
            });
        }
    }
}

impl MaestroServer {
    pub(super) fn schedule_initial_diagnostics(&self, uri: Url, version: i32) -> bool {
        self.initial_diagnostics
            .as_ref()
            .is_some_and(|scheduler| scheduler.schedule(uri, version))
    }
}

#[cfg(test)]
mod tests;
