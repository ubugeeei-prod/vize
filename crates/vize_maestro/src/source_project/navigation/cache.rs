//! Existing snapshot worker admission and configuration cleanup.
use super::{
    Arc, CachedNavigation, Lang, NativeNavigationProject, NavigationRefusal,
    NavigationWorker, Profile, ProgramOptions, QueryFamily, SnapshotRefusal, SourceSnapshot,
};

impl NativeNavigationProject<'_> {
    pub(super) fn worker(
        &self,
        snapshot: Arc<SourceSnapshot>,
    ) -> Result<Arc<NavigationWorker>, NavigationRefusal> {
        self.worker_for(snapshot, false)
    }

    pub(super) fn worker_for(
        &self,
        snapshot: Arc<SourceSnapshot>,
        selected: bool,
    ) -> Result<Arc<NavigationWorker>, NavigationRefusal> {
        self.worker_family(
            snapshot,
            if selected {
                QueryFamily::Selected
            } else {
                QueryFamily::Program
            },
        )
    }

    pub(super) fn worker_family(
        &self,
        snapshot: Arc<SourceSnapshot>,
        family: QueryFamily,
    ) -> Result<Arc<NavigationWorker>, NavigationRefusal> {
        let cache = match family {
            QueryFamily::Program => &self.workers,
            QueryFamily::Selected => &self.selected_workers,
            QueryFamily::Names => &self.names_workers,
        };
        loop {
            let mut workers = cache.lock();
            // A post-mutation hook must acquire this same mutex. If mutation
            // precedes this check, refuse; if it follows, its hook retires the
            // inserted entry. No host guard survives spawning or an await.
            if !self.source.snapshot_is_current(&snapshot) {
                return Err(NavigationRefusal::Host(SnapshotRefusal::Superseded));
            }
            let profile = match (family, self.profile(&snapshot)?) {
                (QueryFamily::Selected, Profile::Vue(configuration)) => {
                    Profile::SelectedVue(configuration)
                }
                (QueryFamily::Names, Profile::Vue(configuration)) => {
                    Profile::TemplateNamesVue(configuration)
                }
                (QueryFamily::Program, profile) => profile,
                _ => return Err(NavigationRefusal::Language),
            };
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

    pub(super) fn profile(&self, snapshot: &SourceSnapshot) -> Result<Profile, NavigationRefusal> {
        match snapshot.language_id() {
            "javascript" => Ok(Profile::Program(ProgramOptions::module(Lang::Js))),
            "typescript" => Ok(Profile::Program(ProgramOptions::module(Lang::Ts))),
            "javascriptreact" => Ok(Profile::Program(ProgramOptions {
                jsx: true,
                ..ProgramOptions::module(Lang::Js)
            })),
            "typescriptreact" => Ok(Profile::Program(ProgramOptions {
                jsx: true,
                ..ProgramOptions::module(Lang::Ts)
            })),
            "vue" if !crate::utils::is_standalone_html_path(snapshot.uri().path()) => self
                .source
                .native_vue_configuration()
                .map(Profile::Vue)
                .ok_or(NavigationRefusal::Configuration),
            _ => Err(NavigationRefusal::Language),
        }
    }

    pub(super) fn checked_response<T>(
        &self,
        result: Result<(Profile, Result<T, NavigationRefusal>), NavigationRefusal>,
    ) -> Result<T, NavigationRefusal> {
        let (profile, response) = result?;
        if let Profile::Vue(configuration)
        | Profile::SelectedVue(configuration)
        | Profile::TemplateNamesVue(configuration) = profile
            && self.source.native_vue_configuration() != Some(configuration)
        {
            return Err(NavigationRefusal::ConfigurationChanged);
        }
        response
    }

    pub(super) fn retire_changed_configuration(&self, uri: &Url) {
        for cache in [&self.workers, &self.selected_workers, &self.names_workers] {
            let retired = {
                let mut workers = cache.lock();
                let current = self.source.native_vue_configuration();
                if workers.get(uri).is_some_and(|cached| {
                matches!(cached.profile, Profile::Vue(configuration) | Profile::SelectedVue(configuration) | Profile::TemplateNamesVue(configuration) if Some(configuration) != current)
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
    }
}
