//! Query leases retain cancellation through final guarded publication.
#![expect(
    clippy::disallowed_types,
    reason = "query leases retain shared registry and original snapshot Arc buffers across await"
)]

use std::future::Future;
use std::sync::Arc;

use crate::document::DocumentStore;
use crate::source_project::{SnapshotRefusal, SourceQuery, SourceQueryResult, SourceSnapshot};

use super::ActiveQueries;

struct QueryLease {
    active: Arc<ActiveQueries>,
    id: u64,
}

impl Drop for QueryLease {
    fn drop(&mut self) {
        let removed = {
            let mut active = self.active.0.lock();
            active
                .iter()
                .position(|query| query.id == self.id)
                .map(|position| active.swap_remove(position))
        };
        // This handle can own the last stored Waker. Its destructor can run
        // arbitrary code, so release the registry lock before dropping it.
        drop(removed);
    }
}

/// One real source query associated with its original project host.
pub struct ProjectQuery<'host> {
    query: SourceQuery,
    documents: &'host DocumentStore,
    lease: QueryLease,
}

impl<'host> ProjectQuery<'host> {
    pub(super) fn new(
        query: SourceQuery,
        documents: &'host DocumentStore,
        active: Arc<ActiveQueries>,
        id: u64,
    ) -> Self {
        Self {
            query,
            documents,
            lease: QueryLease { active, id },
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> &Arc<SourceSnapshot> {
        self.query.snapshot()
    }

    pub async fn run<F, Fut, T>(
        self,
        compute: F,
    ) -> Result<ProjectQueryResult<'host, T>, SnapshotRefusal>
    where
        F: FnOnce(Arc<SourceSnapshot>) -> Fut,
        Fut: Future<Output = T>,
    {
        let Self {
            query,
            documents,
            lease,
        } = self;
        let result = query.run(documents, compute).await?;
        Ok(ProjectQueryResult {
            result,
            documents,
            _lease: lease,
        })
    }
}

/// The lease remains cancellable until ready data is synchronously published.
pub struct ProjectQueryResult<'host, T> {
    result: SourceQueryResult<T>,
    documents: &'host DocumentStore,
    _lease: QueryLease,
}

impl<T> ProjectQueryResult<'_, T> {
    /// Publish only ready response/cache data. The callback must never re-enter
    /// this project or DocumentStore, including reads and edit application.
    /// The underlying actual read guard enforces snapshot freshness; asynchronous
    /// transport and atomic store mutation remain outside this boundary.
    pub fn publish<R>(self, publish: impl FnOnce(T) -> R) -> Result<R, SnapshotRefusal> {
        self.result.publish(self.documents, publish)
    }
}
