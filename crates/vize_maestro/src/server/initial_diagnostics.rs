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

use tower_lsp::lsp_types::Url;

use super::MaestroServer;

/// Let the editor's first interactive request claim Corsa before validation.
/// Initial diagnostics remain prompt, while completion/hover no longer queue
/// behind bridge startup and a full project diagnostic request.
const INTERACTIVE_GRACE: Duration = Duration::from_secs(1);

struct InitialDiagnosticsJob {
    uri: Url,
    version: i32,
    not_before: Instant,
}

pub(super) struct InitialDiagnosticsScheduler {
    sender: Option<mpsc::Sender<InitialDiagnosticsJob>>,
}

impl InitialDiagnosticsScheduler {
    pub(super) fn new(worker: MaestroServer) -> Self {
        let (sender, receiver) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("vize-initial-diagnostics".into())
            .spawn(move || run_worker(&worker, receiver));

        match spawned {
            Ok(_) => Self {
                sender: Some(sender),
            },
            Err(error) => {
                tracing::error!("failed to start initial diagnostics worker: {error}");
                Self { sender: None }
            }
        }
    }

    pub(super) fn schedule(&self, uri: Url, version: i32) -> bool {
        let Some(sender) = &self.sender else {
            return false;
        };
        let job = InitialDiagnosticsJob {
            uri,
            version,
            not_before: Instant::now() + INTERACTIVE_GRACE,
        };
        if sender.send(job).is_err() {
            tracing::warn!("initial diagnostics worker stopped before accepting a document");
            return false;
        }
        true
    }
}

fn run_worker(worker: &MaestroServer, receiver: mpsc::Receiver<InitialDiagnosticsJob>) {
    while let Ok(job) = receiver.recv() {
        let delay = job.not_before.saturating_duration_since(Instant::now());
        if !delay.is_zero() {
            thread::sleep(delay);
        }
        crate::runtime::block_on(worker.publish_diagnostics_if_version(&job.uri, job.version));
    }
}

impl MaestroServer {
    pub(super) fn schedule_initial_diagnostics(&self, uri: Url, version: i32) -> bool {
        self.initial_diagnostics
            .as_ref()
            .is_some_and(|scheduler| scheduler.schedule(uri, version))
    }
}
