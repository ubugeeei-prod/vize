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
use vize_carton::FxHashMap;

use super::MaestroServer;

/// Let the editor's first interactive request claim Corsa before validation.
/// Initial diagnostics remain prompt, while completion/hover no longer queue
/// behind bridge startup and a full project diagnostic request.
const INTERACTIVE_GRACE: Duration = Duration::from_secs(1);
const MAX_PENDING_DOCUMENTS: usize = 64;

struct InitialDiagnosticsJob {
    version: i32,
    not_before: Instant,
    sequence: u64,
}

#[derive(Default)]
struct PendingInitialDiagnostics {
    jobs: FxHashMap<Url, InitialDiagnosticsJob>,
    next_sequence: u64,
}

impl PendingInitialDiagnostics {
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
            },
        );
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

pub(super) struct InitialDiagnosticsScheduler {
    sender: Option<mpsc::SyncSender<()>>,
    #[allow(clippy::disallowed_types)] // Shared only with the single diagnostics worker.
    pending: std::sync::Arc<Mutex<PendingInitialDiagnostics>>,
}

impl InitialDiagnosticsScheduler {
    #[allow(clippy::disallowed_types)] // One bounded queue is shared with its worker thread.
    pub(super) fn new(worker: MaestroServer) -> Self {
        // The channel carries only a wake token. The authoritative queue keeps
        // at most one pending version per URI, so repeated opens cannot build
        // an unbounded FIFO backlog while Corsa is processing another file.
        let (sender, receiver) = mpsc::sync_channel(1);
        let pending = std::sync::Arc::new(Mutex::new(PendingInitialDiagnostics::default()));
        let worker_pending = std::sync::Arc::clone(&pending);
        let spawned = thread::Builder::new()
            .name("vize-initial-diagnostics".into())
            .spawn(move || run_worker(&worker, receiver, &worker_pending));

        match spawned {
            Ok(_) => Self {
                sender: Some(sender),
                pending,
            },
            Err(error) => {
                tracing::error!("failed to start initial diagnostics worker: {error}");
                Self {
                    sender: None,
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
}

fn run_worker(
    worker: &MaestroServer,
    receiver: mpsc::Receiver<()>,
    pending: &Mutex<PendingInitialDiagnostics>,
) {
    while receiver.recv().is_ok() {
        loop {
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
            crate::runtime::block_on(worker.publish_diagnostics_if_version(&uri, job.version));
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
mod tests {
    use std::time::{Duration, Instant};

    use tower_lsp::lsp_types::Url;

    use super::{MAX_PENDING_DOCUMENTS, PendingInitialDiagnostics};

    #[test]
    fn pending_jobs_keep_only_the_newest_version_per_uri() {
        let uri = Url::parse("file:///workspace/App.vue").unwrap();
        let now = Instant::now();
        let mut pending = PendingInitialDiagnostics::default();
        pending.insert(uri.clone(), 1, now + Duration::from_secs(1));
        pending.insert(uri.clone(), 3, now + Duration::from_secs(3));
        pending.insert(uri.clone(), 2, now + Duration::from_secs(2));

        assert_eq!(pending.jobs.len(), 1);
        assert!(pending.take_ready(now + Duration::from_secs(2)).is_none());
        let (ready_uri, ready) = pending.take_ready(now + Duration::from_secs(3)).unwrap();
        assert_eq!(ready_uri, uri);
        assert_eq!(ready.version, 3);
        assert!(pending.jobs.is_empty());
    }

    #[test]
    fn distinct_uris_keep_independent_pending_jobs() {
        let now = Instant::now();
        let first = Url::parse("file:///workspace/First.vue").unwrap();
        let second = Url::parse("file:///workspace/Second.vue").unwrap();
        let mut pending = PendingInitialDiagnostics::default();
        for (uri, version) in [(first, 4), (second, 7)] {
            pending.insert(uri, version, now);
        }

        let mut versions = [
            pending.take_ready(now).unwrap().1.version,
            pending.take_ready(now).unwrap().1.version,
        ];
        versions.sort_unstable();
        assert_eq!(versions, [4, 7]);
        assert!(pending.jobs.is_empty());
    }

    #[test]
    #[allow(clippy::disallowed_macros)] // Test-only URI generation stays local and explicit.
    fn pending_jobs_evict_the_oldest_uri_at_capacity() {
        let now = Instant::now();
        let mut pending = PendingInitialDiagnostics::default();
        for index in 0..MAX_PENDING_DOCUMENTS {
            pending.insert(
                Url::parse(&format!("file:///workspace/{index}.vue")).unwrap(),
                index as i32,
                now,
            );
        }
        let oldest = Url::parse("file:///workspace/0.vue").unwrap();
        let newest = Url::parse("file:///workspace/newest.vue").unwrap();
        pending.insert(newest.clone(), 99, now);

        assert_eq!(pending.jobs.len(), MAX_PENDING_DOCUMENTS);
        assert!(!pending.jobs.contains_key(&oldest));
        assert_eq!(pending.jobs.get(&newest).unwrap().version, 99);
    }
}
