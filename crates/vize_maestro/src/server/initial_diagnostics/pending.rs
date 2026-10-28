//! One coalesced pending descriptor per currently open document.

use super::EDIT_DIAGNOSTIC_DEBOUNCE;
use std::time::{Duration, Instant};
use tower_lsp::lsp_types::Url;
use vize_l0::FxHashMap;

#[derive(Clone)]
pub(super) struct InitialDiagnosticsJob {
    pub(super) cause: DiagnosticsJobCause,
    pub(super) version: i32,
    pub(super) not_before: Instant,
    pub(super) sequence: u64,
    pub(super) sync_pending: bool,
    pub(super) sync_running: bool,
    pub(super) preparation_pending: bool,
    pub(super) sync: Option<super::super::diagnostic_publishing::SyncDiagnostics>,
    pub(super) continuation: Option<super::super::state::InitialConfigurationContinuation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DiagnosticsJobCause {
    Open,
    Edit,
}

#[derive(Default)]
pub(super) struct PendingInitialDiagnostics {
    pub(super) jobs: FxHashMap<Url, InitialDiagnosticsJob>,
    pub(super) next_sequence: u64,
    /// Lane occupancy of the last native pass; unknown until one finishes.
    pub(super) last_pass: Option<Duration>,
}

impl PendingInitialDiagnostics {
    /// The caller owns the pending lock, so a reopen cannot insert a new job between this check and removal.
    pub(super) fn discard_closed(&mut self, uri: &Url, documents: &crate::document::DocumentStore) {
        if !documents.contains(uri) {
            self.jobs.remove(uri);
        }
    }

    pub(super) fn complete(&mut self, uri: &Url, version: i32) {
        if self.jobs.get(uri).is_some_and(|job| job.version <= version) {
            self.jobs.remove(uri);
        }
    }

    pub(super) fn edit_not_before(&self, uri: &Url, now: Instant) -> Instant {
        let cheap = self
            .last_pass
            .is_some_and(|pass| pass < EDIT_DIAGNOSTIC_DEBOUNCE);
        if cheap && !self.jobs.contains_key(uri) {
            now
        } else {
            now + EDIT_DIAGNOSTIC_DEBOUNCE
        }
    }

    pub(super) fn insert(&mut self, uri: Url, version: i32, not_before: Instant) {
        self.insert_with_continuation(
            uri,
            version,
            not_before,
            DiagnosticsJobCause::Open,
            None,
            None,
        );
    }

    pub(super) fn insert_with_continuation(
        &mut self,
        uri: Url,
        version: i32,
        not_before: Instant,
        cause: DiagnosticsJobCause,
        sync: Option<super::super::diagnostic_publishing::SyncDiagnostics>,
        continuation: Option<super::super::state::InitialConfigurationContinuation>,
    ) {
        if self
            .jobs
            .get(&uri)
            .is_some_and(|pending| pending.version > version)
        {
            return;
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

    pub(super) fn take_sync(&mut self) -> Option<(Url, InitialDiagnosticsJob)> {
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

    pub(super) fn complete_sync(&mut self, uri: &Url, sequence: u64) {
        if let Some(job) = self
            .jobs
            .get_mut(uri)
            .filter(|job| job.sequence == sequence)
        {
            job.sync_running = false;
        }
    }

    pub(super) fn take_preparation(&mut self) -> Option<(Url, InitialDiagnosticsJob)> {
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

    pub(super) fn next_not_before(&self) -> Option<Instant> {
        self.jobs
            .values()
            .filter(|job| !job.sync_pending && !job.sync_running)
            .map(|job| job.not_before)
            .min()
    }

    pub(super) fn take_ready(&mut self, now: Instant) -> Option<(Url, InitialDiagnosticsJob)> {
        let uri = self
            .jobs
            .iter()
            .filter(|(_, job)| !job.sync_pending && !job.sync_running && job.not_before <= now)
            .min_by_key(|(_, job)| job.not_before)
            .map(|(uri, _)| uri.clone())?;
        self.jobs.remove(&uri).map(|job| (uri, job))
    }
}
