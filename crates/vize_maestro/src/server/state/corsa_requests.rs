//! One native editor transaction, including opening and mapping its sources.

use futures::lock::MutexGuard;
use std::sync::atomic::Ordering;

use super::ServerState;

#[derive(Clone, Copy)]
pub(crate) struct CorsaRequestStamp {
    documents: Option<u64>,
    environment: u64,
    stable: bool,
}

impl CorsaRequestStamp {
    pub(crate) fn is_current(self, state: &ServerState) -> bool {
        self.stable
            && state.corsa_environment_changes.load(Ordering::Acquire) == 0
            && self.documents.is_some()
            && self.documents == state.documents.stable_revision()
            && self.environment == state.corsa_environment_revision.load(Ordering::Acquire)
    }
}

pub(crate) struct CorsaRequestScope<'a> {
    state: &'a ServerState,
    _guard: MutexGuard<'a, ()>,
    stamp: CorsaRequestStamp,
}

impl CorsaRequestScope<'_> {
    /// Refuse a whole result when a root, dependency, project, or backend moved.
    /// The mutex prevents another native transaction replacing virtual sources
    /// between opening this request's sources and mapping its native reply.
    pub(crate) fn is_current(&self) -> bool {
        self.stamp.is_current(self.state)
    }

    pub(crate) fn stamp(&self) -> CorsaRequestStamp {
        self.stamp
    }
}

impl ServerState {
    /// Acquire inside the cancellable handler, before taking source snapshots.
    /// Syntax-only requests, notifications, cancellation and shutdown do not
    /// acquire this lock. No synchronous map/config guard survives acquisition.
    pub(crate) async fn corsa_request_scope(&self) -> CorsaRequestScope<'_> {
        let guard = self.corsa_request_lock.lock().await;
        let stable = self.corsa_environment_changes.load(Ordering::Acquire) == 0;
        let environment = self.corsa_environment_revision.load(Ordering::Acquire);
        CorsaRequestScope {
            state: self,
            _guard: guard,
            stamp: CorsaRequestStamp {
                documents: self.documents.stable_revision(),
                environment,
                stable: stable && self.corsa_environment_changes.load(Ordering::Acquire) == 0,
            },
        }
    }

    pub(super) fn invalidate_corsa_request_environment(&self) {
        self.corsa_environment_revision
            .fetch_add(1, Ordering::Release);
    }

    pub(super) fn corsa_environment_change(&self) -> CorsaEnvironmentChange<'_> {
        self.corsa_environment_changes
            .fetch_add(1, Ordering::AcqRel);
        self.invalidate_corsa_request_environment();
        CorsaEnvironmentChange(self)
    }
}

pub(super) struct CorsaEnvironmentChange<'a>(&'a ServerState);

impl Drop for CorsaEnvironmentChange<'_> {
    fn drop(&mut self) {
        self.0.invalidate_corsa_request_environment();
        self.0
            .corsa_environment_changes
            .fetch_sub(1, Ordering::Release);
    }
}
