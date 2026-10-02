//! Cooperative cancellation and final guarded host publication.
#![expect(
    clippy::disallowed_types,
    reason = "an owned host snapshot must outlive guard-free async query suspension"
)]

use std::future::Future;
use std::sync::Arc;
use std::task::Poll;

use futures::future::{AbortHandle, AbortRegistration, Abortable};
use tower_lsp::lsp_types::Url;

use crate::document::DocumentStore;

use super::{SnapshotRefusal, SourceSnapshot, SourceSnapshotCache};

/// One query tied to a retained real snapshot and a genuine future cancellation pair.
pub struct SourceQuery {
    snapshot: Arc<SourceSnapshot>,
    registration: AbortRegistration,
}

impl SourceSnapshotCache {
    pub fn begin_query(
        &self,
        documents: &DocumentStore,
        uri: &Url,
    ) -> Result<(SourceQuery, AbortHandle), SnapshotRefusal> {
        let snapshot = self.capture(documents, uri)?;
        let (cancel, registration) = AbortHandle::new_pair();
        Ok((
            SourceQuery {
                snapshot,
                registration,
            },
            cancel,
        ))
    }
}

impl SourceQuery {
    #[must_use]
    pub fn snapshot(&self) -> &Arc<SourceSnapshot> {
        &self.snapshot
    }

    /// Run without a document guard across an await. Dropping this future also
    /// drops its computation, as required by the tower-lsp cancellation layer.
    /// Explicit AbortHandle cancellation wakes a pending computation. Synchronous
    /// work is cooperative and cannot be preempted during one poll.
    pub async fn run<F, Fut, T>(
        self,
        documents: &DocumentStore,
        compute: F,
    ) -> Result<SourceQueryResult<T>, SnapshotRefusal>
    where
        F: FnOnce(Arc<SourceSnapshot>) -> Fut,
        Fut: Future<Output = T>,
    {
        let Self {
            snapshot,
            registration,
        } = self;
        let cancel = registration.handle();
        let computation = async {
            snapshot.check_current(documents)?;
            yield_to_cancellation().await;
            snapshot.check_current(documents)?;
            if cancel.is_aborted() {
                return Err(SnapshotRefusal::Cancelled);
            }
            let value = compute(Arc::clone(&snapshot)).await;
            if cancel.is_aborted() {
                return Err(SnapshotRefusal::Cancelled);
            }
            snapshot.check_current(documents)?;
            Ok(SourceQueryResult {
                snapshot,
                value,
                cancel,
            })
        };
        Abortable::new(computation, registration)
            .await
            .map_err(|_| SnapshotRefusal::Cancelled)?
    }
}

/// Completed work still needs a fresh guarded publication check.
pub struct SourceQueryResult<T> {
    snapshot: Arc<SourceSnapshot>,
    value: T,
    cancel: AbortHandle,
}

impl<T> SourceQueryResult<T> {
    /// Perform the synchronous response/cache publication while the actual
    /// current document is read-locked. The callback must not await or re-enter
    /// DocumentStore in any way, including reads or nested apply_edits. A queued
    /// writer can make recursive reads deadlock. Compute native edit output first;
    /// publish only ready synchronous response/cache data inside this callback.
    /// Async transport and atomic document mutation remain outside this boundary.
    pub fn publish<R>(
        self,
        documents: &DocumentStore,
        publish: impl FnOnce(T) -> R,
    ) -> Result<R, SnapshotRefusal> {
        let Self {
            snapshot,
            value,
            cancel,
        } = self;
        snapshot.with_current(documents, |_| {
            if cancel.is_aborted() {
                Err(SnapshotRefusal::Cancelled)
            } else {
                Ok(publish(value))
            }
        })?
    }
}

async fn yield_to_cancellation() {
    let mut yielded = false;
    futures::future::poll_fn(|context| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}
