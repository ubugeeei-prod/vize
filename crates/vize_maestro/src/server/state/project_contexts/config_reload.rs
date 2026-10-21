//! Await root-local admission before consuming a bounded config worker.
use super::paths::physical_path;
use super::{Arc, Path, PathBuf, ProjectContextKey, ServerState, Url};
use tower_lsp::lsp_types::{FileEvent, MessageType, ShowMessageParams};

/// One checked Vite evaluation's post-read repair baseline, shared by its
/// full-key Canon owners. This never supplies consumed-byte authority or a
/// current validation sample, and is dropped with the local authored attempt.
pub(super) struct ConfigurationPostRead {
    reported: Vec<PathBuf>,
    known: super::FxHashSet<PathBuf>,
    common: Option<vize_canon::ConfigurationObservations>,
}

impl ConfigurationPostRead {
    pub(super) fn new(reported: &[PathBuf]) -> Self {
        Self {
            reported: reported.to_vec(),
            known: reported.iter().cloned().collect(),
            common: None,
        }
    }

    pub(super) fn capture(&mut self, applied: &[PathBuf]) -> vize_canon::ConfigurationObservations {
        // Capture lazily at the first successful Canon owner's original
        // post-read boundary. Later owners retain this earliest baseline.
        let mut observations = self
            .common
            .get_or_insert_with(|| vize_canon::ConfigurationObservations::capture(&self.reported))
            .clone();
        let specific = applied
            .iter()
            .filter(|path| !self.known.contains(*path))
            .cloned()
            .collect::<Vec<_>>();
        // Canon-specific paths (and their link ancestors) are still captured
        // separately for each owner, retaining conflicts with consumed inputs.
        observations.extend(&vize_canon::ConfigurationObservations::capture(&specific));
        observations
    }
}

pub(super) struct ConfigEventScope {
    pub(super) affected: Vec<ProjectContextKey>,
    pub(super) confirmations: Vec<(ProjectContextKey, vize_canon::ConfigurationObservations)>,
    pub(super) marker_paths: Vec<PathBuf>,
}

impl ConfigEventScope {
    pub(super) fn for_owners(affected: Vec<ProjectContextKey>) -> Self {
        Self {
            affected,
            confirmations: Vec::new(),
            marker_paths: Vec::new(),
        }
    }

    pub(super) fn for_topology(
        affected: Vec<ProjectContextKey>,
        marker_paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            affected,
            confirmations: Vec::new(),
            marker_paths,
        }
    }

    pub(super) fn retain_root(&mut self, root: &Path) {
        self.affected
            .retain(|key| physical_path(&key.root) == physical_path(root));
        self.confirmations
            .retain(|(key, _)| physical_path(&key.root) == physical_path(root));
        self.marker_paths
            .retain(|path| path.starts_with(physical_path(root)));
    }
}

#[derive(Default)]
pub(crate) struct ProjectConfigChanges {
    pub documents: Vec<(Url, i32)>,
    pub errors: Vec<ShowMessageParams>,
    /// Roots actually checked in this operation, including rejected attempts.
    pub(crate) checked_roots: Vec<PathBuf>,
    pub(super) rejected: Vec<(ProjectContextKey, Vec<Url>)>,
    pub(super) configuration_events: Vec<Url>,
}

impl ProjectConfigChanges {
    /// A rejected dependency must not invalidate the retained native program.
    /// Other events in the same notification still reach this owner.
    pub(crate) fn accepted_file_events(
        &self,
        state: &ServerState,
        changes: &[FileEvent],
    ) -> Vec<FileEvent> {
        let key = state.project_contexts.key.read().clone().or_else(|| {
            state
                .project_contexts
                .boundary
                .read()
                .clone()
                .or_else(|| state.get_workspace_root())
                .map(ProjectContextKey::from)
        });
        changes
            .iter()
            .filter(|change| {
                if self.rejected.iter().any(|(rejected, dependencies)| {
                    key.as_ref() == Some(rejected) && dependencies.contains(&change.uri)
                }) {
                    return false;
                }
                if !self.configuration_events.contains(&change.uri) {
                    return true;
                }
                // A sibling's config is not a disk mutation of this project. Keep
                // ordinary source imports of that same file observable, including
                // cross-package imports recorded by the open-importer index.
                let dependency = change.uri.to_file_path().is_ok_and(|path| {
                    state
                        .project_contexts
                        .config_dependencies
                        .read()
                        .iter()
                        .any(|dependency| physical_path(dependency) == physical_path(&path))
                });
                dependency
                    || !crate::server::importers::open_typecheck_dependents(state, &change.uri)
                        .is_empty()
            })
            .cloned()
            .collect()
    }
}

pub(super) fn reload_error(root: &std::path::Path, changes: &[Url]) -> ShowMessageParams {
    let root = Url::from_file_path(root)
        .map(|root| vize_l0::cstr!("{root}"))
        .unwrap_or_else(|()| vize_l0::cstr!("{}", root.display()));
    // One file reported through a symlink and through its physical path is
    // one change (#9686).
    let mut seen = Vec::new();
    let changed = changes
        .iter()
        .filter(|uri| {
            let Ok(path) = uri.to_file_path() else {
                return true;
            };
            let physical = physical_path(&path);
            let first = !seen.contains(&physical);
            seen.push(physical);
            first
        })
        .map(Url::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let cause = if changes.is_empty() {
        vize_l0::String::default()
    } else {
        vize_l0::cstr!(" after changes to {changed}")
    };
    ShowMessageParams {
        typ: MessageType::ERROR,
        message: vize_l0::cstr!("Vize could not reload project configuration for {root}{cause}. Keeping the last good configuration.").into(),
    }
}

pub(super) fn worker_unavailable_error() -> ShowMessageParams {
    ShowMessageParams { typ: MessageType::ERROR, message: "Vize could not reload project configuration because the configuration worker is unavailable. Keeping the last good configuration.".into() }
}

impl ServerState {
    pub(super) fn config_reload_gates(
        &self,
        roots: &[PathBuf],
    ) -> Vec<Arc<futures::lock::Mutex<()>>> {
        let mut admission = self.project_contexts.reload_admission.lock();
        admission.retain(|_, gate| gate.strong_count() != 0);
        roots
            .iter()
            .map(|root| {
                if let Some(gate) = admission.get(root).and_then(std::sync::Weak::upgrade) {
                    return gate;
                }
                let gate = Arc::new(futures::lock::Mutex::new(()));
                admission.insert(root.clone(), Arc::downgrade(&gate));
                gate
            })
            .collect()
    }

    pub(crate) async fn observe_project_config_events_async(
        self: &Arc<Self>,
        changes: Vec<FileEvent>,
    ) -> ProjectConfigChanges {
        let mut roots = self
            .project_config_event_scope(&changes)
            .affected
            .into_iter()
            .map(|key| key.root)
            .collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        let results = futures::future::join_all(roots.into_iter().map(|root| {
            let changes = &changes;
            async move {
                let gates = self.config_reload_gates(std::slice::from_ref(&root));
                let Some(gate) = gates.first() else {
                    return ProjectConfigChanges::default();
                };
                let admission = gate.clone().lock_owned().await;
                let mut fresh = self.project_config_event_scope(changes);
                if !fresh.affected.iter().any(|key| key.root == root) {
                    return ProjectConfigChanges::default();
                }
                fresh.retain_root(&root);
                let state = self.clone();
                let worker_changes = changes.clone();
                Self::project_background(move || {
                    // The queued job owns admission even if its requesting
                    // notification is canceled after worker submission.
                    let _admission = admission;
                    state.apply_project_config_events(&worker_changes, fresh)
                })
                .await
                .unwrap_or_else(|| self.config_worker_unavailable(&root, changes))
            }
        }))
        .await;
        let mut result = ProjectConfigChanges::default();
        for next in results {
            result.documents.extend(next.documents);
            result.errors.extend(next.errors);
            result.checked_roots.extend(next.checked_roots);
            result.rejected.extend(next.rejected);
            result
                .configuration_events
                .extend(next.configuration_events);
        }
        result.documents.sort();
        result.documents.dedup();
        result
    }

    pub(super) fn admits_new_config_root(&self, root: &std::path::Path) -> bool {
        self.is_primary_boundary(root)
            || self
                .workspace_root_paths()
                .iter()
                .any(|workspace| physical_path(root).starts_with(physical_path(workspace)))
    }

    pub(super) fn config_worker_unavailable(
        &self,
        root: &std::path::Path,
        changes: &[FileEvent],
    ) -> ProjectConfigChanges {
        let affected = self.project_config_event_scope(changes).affected;
        if !affected.iter().any(|key| key.root == root) {
            return ProjectConfigChanges::default();
        }
        let contexts = self.project_contexts.contexts.lock();
        let registered = contexts.keys().any(|key| key.root == root);
        if self.project_routing_shutting_down()
            || (!registered && !self.admits_new_config_root(root))
        {
            return ProjectConfigChanges::default();
        }
        let mut keys = contexts
            .iter()
            .filter(|(key, _)| key.root == root && affected.contains(key))
            .map(|(key, context)| (key.clone(), context.config_dependencies()))
            .collect::<Vec<_>>();
        if self.is_primary_boundary(root)
            && affected.contains(&ProjectContextKey::from(root.to_path_buf()))
            && !keys.iter().any(|(key, _)| key.is_root())
        {
            keys.push((
                root.to_path_buf().into(),
                self.project_contexts.config_dependencies.read().clone(),
            ));
        }
        if keys.is_empty() {
            keys.push((root.to_path_buf().into(), Vec::new()));
        }
        let roots = self.workspace_root_paths();
        let mut result = ProjectConfigChanges {
            errors: vec![worker_unavailable_error()],
            ..Default::default()
        };
        for (key, dependencies) in keys {
            let mut rejected = changes
                .iter()
                .filter(|change| {
                    change.uri.to_file_path().is_ok_and(|path| {
                        let path = physical_path(&path);
                        dependencies
                            .iter()
                            .any(|dependency| physical_path(dependency) == path)
                            || (key.is_root()
                                && super::is_config_marker(&path)
                                && path.starts_with(physical_path(root))
                                && !super::watched::excluded_config_path(&path, &roots))
                    })
                })
                .map(|change| change.uri.clone())
                .collect::<Vec<_>>();
            rejected.sort();
            rejected.dedup();
            if let Some(context) = contexts.get(&key) {
                context.record_reload_failure();
            } else if key.is_root() && self.is_primary_boundary(root) {
                self.project_contexts
                    .config_reload_failed
                    .store(true, super::Ordering::Release);
            }
            self.project_contexts
                .pending_config_reloads
                .lock()
                .insert(key.clone());
            result.configuration_events.extend(rejected.clone());
            result.rejected.push((key, rejected));
        }
        result
    }

    pub(super) async fn await_project_config_reload(&self, root: &std::path::Path) {
        let gate = self
            .project_contexts
            .reload_admission
            .lock()
            .get(root)
            .and_then(std::sync::Weak::upgrade);
        if let Some(gate) = gate {
            let _admission = gate.lock_owned().await;
        }
    }

    #[cfg(test)]
    pub(crate) fn observe_project_config_events(
        self: &Arc<Self>,
        changes: &[FileEvent],
    ) -> Vec<(Url, i32)> {
        let scope = self.project_config_event_scope(changes);
        let mut roots = scope
            .affected
            .iter()
            .map(|key| key.root.clone())
            .collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        let gates = self.config_reload_gates(&roots);
        let _guards = gates
            .iter()
            .map(|gate| {
                gate.try_lock_owned()
                    .unwrap_or_else(|| futures::executor::block_on(gate.clone().lock_owned()))
            })
            .collect::<Vec<_>>();
        self.observe_project_config_events_checked(changes)
            .documents
    }

    #[cfg(test)]
    pub(crate) fn observe_project_config_events_checked(
        self: &Arc<Self>,
        changes: &[FileEvent],
    ) -> ProjectConfigChanges {
        self.apply_project_config_events(changes, self.project_config_event_scope(changes))
    }
}

pub(super) fn known_config_events(
    root: &Path,
    dependencies: &[PathBuf],
    markers: &[PathBuf],
    changes: &[Url],
) -> Vec<Url> {
    let mut changed = changes
        .iter()
        .filter(|change| {
            change.to_file_path().is_ok_and(|path| {
                let path = physical_path(&path);
                dependencies
                    .iter()
                    .any(|dependency| physical_path(dependency) == path)
                    || (markers.contains(&path) && path.starts_with(physical_path(root)))
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    changed.sort();
    changed.dedup();
    changed
}
