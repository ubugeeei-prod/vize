//! Opt-in local queries over original Program or SFC owners per host snapshot.
#![expect(
    clippy::disallowed_types,
    reason = "async host queries retain immutable Arc snapshots and owned responses"
)]

mod coordinates;
pub(in crate::source_project) mod profile;
mod retained;
#[cfg(test)]
mod tests;
mod worker;

use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tower_lsp::lsp_types::{Location, Position, Url};
use vize_l0::FxHashMap;
use vize_l2::file::{FileIssue, PositionQueryError};

use super::{SnapshotRefusal, SourceQueryProject, SourceSnapshot};
use profile::Profile;
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
    Busy,
    Capacity,
    WorkerUnavailable,
    Configuration,
    ConfigurationChanged,
    SfcProducer(Vec<vize_l1_to_l2::native_file::NativeSfcIssue>),
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
    live: Arc<AtomicUsize>,
}

impl<'host> NativeNavigationProject<'host> {
    pub fn new(source: SourceQueryProject<'host>) -> Self {
        Self {
            source,
            workers: Mutex::new(FxHashMap::default()),
            live: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Called after the genuine host update, never instead of that mutation.
    pub fn notify_host_change(&self, uri: &Url) {
        self.source.notify_host_change(uri);
        let snapshot = self
            .workers
            .lock()
            .get(uri)
            .map(|cached| Arc::clone(&cached.snapshot));
        if let Some(snapshot) = snapshot
            && !self.source.snapshot_is_current(&snapshot)
        {
            // Preserve a fresh worker inserted between the host change and hook.
            let retired = {
                let mut workers = self.workers.lock();
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

    fn worker(
        &self,
        snapshot: Arc<SourceSnapshot>,
    ) -> Result<Arc<NavigationWorker>, NavigationRefusal> {
        loop {
            let mut workers = self.workers.lock();
            // A post-mutation hook must acquire this same mutex. If mutation
            // precedes this check, refuse; if it follows, its hook retires the
            // inserted entry. No host guard survives spawning or an await.
            if !self.source.snapshot_is_current(&snapshot) {
                return Err(NavigationRefusal::Host(SnapshotRefusal::Superseded));
            }
            let profile = self.profile(&snapshot)?;
            if let Some(cached) = workers.get(snapshot.uri()) {
                if Arc::ptr_eq(&cached.snapshot, &snapshot) && cached.profile == profile {
                    if let Ok(worker) = &cached.result
                        && !worker.belongs_to(&snapshot)
                    {
                        return Err(NavigationRefusal::Projection);
                    }
                    return cached.result.clone();
                }
                if cached.snapshot.revision() > snapshot.revision() {
                    return Err(NavigationRefusal::Host(SnapshotRefusal::Superseded));
                }
            }
            if let Some(retired) = workers.remove(snapshot.uri()) {
                drop(workers);
                if let Ok(worker) = retired.result {
                    worker.retire();
                }
                continue;
            }
            // Serialize misses, but parsing and response waits run outside locks.
            // Transient capacity/spawn refusals are retryable, never memoized.
            let result =
                NavigationWorker::spawn(Arc::clone(&snapshot), Arc::clone(&self.live), profile);
            if matches!(
                result,
                Err(NavigationRefusal::Capacity | NavigationRefusal::WorkerUnavailable)
            ) {
                return result;
            }
            workers.insert(
                snapshot.uri().clone(),
                CachedNavigation {
                    snapshot,
                    profile,
                    result: result.clone(),
                },
            );
            return result;
        }
    }

    fn profile(&self, snapshot: &SourceSnapshot) -> Result<Profile, NavigationRefusal> {
        match snapshot.language_id() {
            "javascript" => Ok(Profile::Program(vize_l1::embed::Lang::Js)),
            "typescript" => Ok(Profile::Program(vize_l1::embed::Lang::Ts)),
            "vue" if !crate::utils::is_standalone_html_path(snapshot.uri().path()) => self
                .source
                .native_vue_configuration()
                .map(Profile::Vue)
                .ok_or(NavigationRefusal::Configuration),
            _ => Err(NavigationRefusal::Language),
        }
    }

    fn checked_response<T>(
        &self,
        result: Result<(Profile, Result<T, NavigationRefusal>), NavigationRefusal>,
    ) -> Result<T, NavigationRefusal> {
        let (profile, response) = result?;
        if let Profile::Vue(configuration) = profile
            && self.source.native_vue_configuration() != Some(configuration)
        {
            return Err(NavigationRefusal::ConfigurationChanged);
        }
        response
    }

    fn retire_changed_configuration(&self, uri: &Url) {
        let retired = {
            let mut workers = self.workers.lock();
            let current = self.source.native_vue_configuration();
            if workers.get(uri).is_some_and(|cached| {
                matches!(cached.profile, Profile::Vue(configuration) if Some(configuration) != current)
            }) {
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
        let retired = core::mem::take(self.workers.get_mut());
        for cached in retired.into_values() {
            if let Ok(worker) = cached.result {
                worker.retire();
            }
        }
    }
}
