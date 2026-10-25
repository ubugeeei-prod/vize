//! Opt-in local queries over original Program or SFC owners per host snapshot.
#![expect(
    clippy::disallowed_types,
    reason = "async host queries retain immutable Arc snapshots and owned responses"
)]

mod cache;
mod coordinates;
mod highlights;
mod intrinsic;
mod linked;
mod module_links;
pub(in crate::source_project) mod names;
pub(in crate::source_project) mod profile;
mod retained;
mod selected;
#[cfg(test)]
mod tests;
mod worker;

use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tower_lsp::lsp_types::{Location, Position, Url};
use vize_l0::FxHashMap;
use vize_l1::embed::{Lang, syntax::ProgramOptions};
use vize_l2::file::{FileIssue, PositionQueryError};

use super::{SnapshotRefusal, SourceQueryProject, SourceSnapshot};
pub use intrinsic::IntrinsicTagTarget;
pub(crate) use module_links::ModuleDocumentLinkError;
use profile::{Profile, QueryFamily};
use worker::NavigationWorker;

/// Unsupported observations remain refusals; no legacy query is substituted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationRefusal {
    Host(SnapshotRefusal),
    Language,
    Syntax,
    Producer(Vec<FileIssue>),
    Projection,
    Position,
    Query(PositionQueryError),
    NativeQuery(vize_l1_to_l2::native_file::NativePositionQueryError),
    TemplateQuery(vize_l2::file::TemplateQueryError),
    SelectedSfcProducer(Vec<vize_l1_to_l2::native_file::NativeSelectedSfcIssue>),
    Busy,
    Capacity,
    WorkerUnavailable,
    Configuration,
    ConfigurationChanged,
    NamesRouteChanged,
    SfcProducer(Vec<vize_l1_to_l2::native_file::NativeSfcIssue>),
    TemplateNamesProducer,
    TemplateFrameNames(vize_l1::container::vue::NativeTemplateFrameNameRefusal),
    ElementNames(vize_l1::markup::NativeElementNameRefusal),
}

struct CachedNavigation {
    snapshot: Arc<SourceSnapshot>,
    profile: Profile,
    result: Result<Arc<NavigationWorker>, NavigationRefusal>,
}

/// Coarse snapshot workers retain genuine original syntax and File locally.
/// Only commands and owned responses cross threads. Native owners never need
/// Send/Sync or survive a snapshot replacement; active thread slots are bounded.
pub struct NativeNavigationProject<'host> {
    source: SourceQueryProject<'host>,
    workers: Mutex<FxHashMap<Url, CachedNavigation>>,
    selected_workers: Mutex<FxHashMap<Url, CachedNavigation>>,
    names_workers: Mutex<FxHashMap<Url, CachedNavigation>>,
    live: Arc<AtomicUsize>,
}

impl<'host> NativeNavigationProject<'host> {
    pub fn new(source: SourceQueryProject<'host>) -> Self {
        Self {
            source,
            workers: Mutex::new(FxHashMap::default()),
            selected_workers: Mutex::new(FxHashMap::default()),
            names_workers: Mutex::new(FxHashMap::default()),
            live: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) fn mark_retired(&self) {
        self.source.mark_retired();
    }

    pub(crate) fn retire(&self) {
        self.source.retire();
        for cache in [&self.workers, &self.selected_workers, &self.names_workers] {
            let retired = core::mem::take(&mut *cache.lock());
            for cached in retired.into_values() {
                if let Ok(worker) = cached.result {
                    worker.retire();
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn cached_program_worker_for_test(
        &self,
        uri: &Url,
    ) -> Option<(usize, (usize, usize))> {
        let workers = self.workers.lock();
        let worker = workers.get(uri)?.result.as_ref().ok()?;
        Some((Arc::as_ptr(worker) as usize, worker.counts()))
    }

    /// Called after the genuine host update, never instead of that mutation.
    pub fn notify_host_change(&self, uri: &Url) {
        self.source.notify_host_change(uri);
        for cache in [&self.workers, &self.selected_workers, &self.names_workers] {
            let snapshot = cache
                .lock()
                .get(uri)
                .map(|cached| Arc::clone(&cached.snapshot));
            if let Some(snapshot) = snapshot
                && !self.source.snapshot_is_current(&snapshot)
            {
                // Preserve a fresh worker inserted between the host change and hook.
                let retired = {
                    let mut workers = cache.lock();
                    if workers
                        .get(uri)
                        .is_some_and(|cached| Arc::ptr_eq(&cached.snapshot, &snapshot))
                    {
                        workers.remove(uri)
                    } else {
                        None
                    }
                };
                if let Some(CachedNavigation {
                    result: Ok(worker), ..
                }) = retired
                {
                    worker.retire();
                }
            }
        }
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
            .run(|snapshot| async move {
                let worker = self.worker(snapshot)?;
                Ok((worker.profile(), worker.definition(position).await))
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        let result = ready
            .publish(|response| self.checked_response(response))
            .map_err(NavigationRefusal::Host)?;
        if matches!(result, Err(NavigationRefusal::ConfigurationChanged)) {
            self.retire_changed_configuration(uri);
        }
        result
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
                let worker = self.worker(snapshot)?;
                Ok((
                    worker.profile(),
                    worker.references(position, include_declaration).await,
                ))
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        let result = ready
            .publish(|response| self.checked_response(response))
            .map_err(NavigationRefusal::Host)?;
        if matches!(result, Err(NavigationRefusal::ConfigurationChanged)) {
            self.retire_changed_configuration(uri);
        }
        result
    }
}

impl Drop for NativeNavigationProject<'_> {
    fn drop(&mut self) {
        self.retire();
    }
}
