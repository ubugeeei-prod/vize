//! Opt-in local JS/TS/JSX/TSX queries over one original Program per host snapshot.
#![expect(
    clippy::disallowed_types,
    reason = "async host queries retain immutable Arc snapshots and owned response summaries"
)]

mod coordinates;
mod summary;
#[cfg(test)]
mod tests;

use parking_lot::Mutex;
use std::sync::Arc;
use tower_lsp::lsp_types::{Location, Position, Url};
use vize_l0::FxHashMap;
use vize_l2::file::FileIssue;

use super::{SnapshotRefusal, SourceQueryProject, SourceSnapshot};
use summary::NavigationSummary;

/// Unsupported observations remain refusals; no legacy query is substituted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationRefusal {
    Host(SnapshotRefusal),
    Language,
    Syntax,
    Producer(Vec<FileIssue>),
    Projection,
    Position,
}

struct CachedNavigation {
    snapshot: Arc<SourceSnapshot>,
    result: Result<Arc<NavigationSummary>, NavigationRefusal>,
}

/// Response summaries are consumer data, not retained level AST/artifacts.
/// The same physical snapshot is required for a cache hit. Arena-owned original
/// Program/File owners coexist during construction and never cross an await.
pub struct NativeNavigationProject<'host> {
    source: SourceQueryProject<'host>,
    summaries: Mutex<FxHashMap<Url, CachedNavigation>>,
}

impl<'host> NativeNavigationProject<'host> {
    pub fn new(source: SourceQueryProject<'host>) -> Self {
        Self {
            source,
            summaries: Mutex::new(FxHashMap::default()),
        }
    }

    /// Called after the genuine host update, never instead of that mutation.
    pub fn notify_host_change(&self, uri: &Url) {
        self.source.notify_host_change(uri);
        // This independent mutex is never acquired under a host read guard or
        // ready publication callback. Recheck each retained snapshot against
        // the live store, preserving any fresh entry created before this hook.
        self.summaries
            .lock()
            .retain(|key, cached| key != uri || self.source.snapshot_is_current(&cached.snapshot));
    }

    fn summary(
        &self,
        snapshot: Arc<SourceSnapshot>,
    ) -> Result<Arc<NavigationSummary>, NavigationRefusal> {
        let mut summaries = self.summaries.lock();
        if let Some(cached) = summaries.get(snapshot.uri())
            && Arc::ptr_eq(&cached.snapshot, &snapshot)
        {
            return cached.result.clone();
        }
        // Serialize cache misses for this project; parsing is synchronous and
        // no store/project read occurs while this independent mutex is held.
        let result = NavigationSummary::build(Arc::clone(&snapshot)).map(Arc::new);
        summaries.insert(
            snapshot.uri().clone(),
            CachedNavigation {
                snapshot,
                result: result.clone(),
            },
        );
        result
    }

    pub async fn definition(
        &self,
        uri: &Url,
        position: Position,
    ) -> Result<Option<Location>, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let ready = query
            .run(|snapshot| async move { self.summary(snapshot)?.definition(position) })
            .await
            .map_err(NavigationRefusal::Host)?;
        ready
            .publish(|response| response)
            .map_err(NavigationRefusal::Host)?
    }

    pub async fn references(
        &self,
        uri: &Url,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<Location>, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let ready = query
            .run(|snapshot| async move {
                self.summary(snapshot)?
                    .references(position, include_declaration)
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        ready
            .publish(|response| response)
            .map_err(NavigationRefusal::Host)?
    }
}
