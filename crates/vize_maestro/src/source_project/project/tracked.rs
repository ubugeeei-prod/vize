//! Query leases retain cancellation through final guarded publication.
#![expect(
    clippy::disallowed_types,
    reason = "query leases retain shared registry and original snapshot Arc buffers across await"
)]

use std::future::Future;
use std::sync::Arc;
#[cfg(feature = "experimental-source-navigation")]
mod modules;
#[cfg(feature = "experimental-source-navigation")]
mod targets;
#[cfg(feature = "experimental-source-navigation")]
pub(crate) use modules::ModuleLinkPublicationError;

use crate::source_project::{SnapshotRefusal, SourceQuery, SourceQueryResult, SourceSnapshot};

use super::{ActiveQueries, host::DocumentHost};

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
    host: DocumentHost<'host>,
    lease: QueryLease,
}

impl<'host> ProjectQuery<'host> {
    pub(super) fn new(
        query: SourceQuery,
        host: DocumentHost<'host>,
        active: Arc<ActiveQueries>,
        id: u64,
    ) -> Self {
        Self {
            query,
            host,
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
        let Self { query, host, lease } = self;
        let result = query.run(host.documents(), compute).await?;
        Ok(ProjectQueryResult {
            result,
            host,
            _lease: lease,
        })
    }
}

/// The lease remains cancellable until ready data is synchronously published.
pub struct ProjectQueryResult<'host, T> {
    result: SourceQueryResult<T>,
    host: DocumentHost<'host>,
    _lease: QueryLease,
}

impl<T> ProjectQueryResult<'_, T> {
    /// Publish only ready response/cache data. The callback must never re-enter
    /// this project or DocumentStore, including reads and edit application.
    /// The underlying actual read guard enforces snapshot freshness; asynchronous
    /// transport and atomic store mutation remain outside this boundary.
    pub fn publish<R>(self, publish: impl FnOnce(T) -> R) -> Result<R, SnapshotRefusal> {
        self.result.publish(self.host.documents(), publish)
    }
}
