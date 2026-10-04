//! Authentic configuration tickets admit only the existing Names worker map.
use super::{
    Arc, CachedNavigation, NativeNavigationProject, NavigationRefusal, NavigationWorker, Profile,
    SnapshotRefusal, SourceSnapshot,
};
use crate::server::{
    NativeLinkedNamesTicket, NativeNamesConfigurationError, NativeNamesParserTicket,
};

pub(in crate::source_project) enum NamesConfiguration {
    Parser(NativeNamesParserTicket),
    Linked(NativeLinkedNamesTicket),
}
impl From<NativeNamesConfigurationError> for NavigationRefusal {
    fn from(error: NativeNamesConfigurationError) -> Self {
        match error {
            NativeNamesConfigurationError::ParserChanged => Self::ConfigurationChanged,
            NativeNamesConfigurationError::RouteChanged => Self::NamesRouteChanged,
        }
    }
}

impl NativeNavigationProject<'_> {
    pub(super) fn names_worker(
        &self,
        snapshot: Arc<SourceSnapshot>,
        ticket: &NamesConfiguration,
    ) -> Result<Arc<NavigationWorker>, NavigationRefusal> {
        loop {
            let mut workers = self.names_workers.lock();
            if !self.source.snapshot_is_current(&snapshot) {
                return Err(NavigationRefusal::Host(SnapshotRefusal::Superseded));
            }
            if snapshot.language_id() != "vue"
                || crate::utils::is_standalone_html_path(snapshot.uri().path())
            {
                return Err(NavigationRefusal::Language);
            }
            // Map→configuration; the source guard has already been released.
            // The same genuine captured parser settings select this owner.
            let profile = self
                .source
                .with_names_configuration(ticket, Profile::TemplateNamesVue)?;
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
            // No configuration/document guard survives spawning or parsing.
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
}
