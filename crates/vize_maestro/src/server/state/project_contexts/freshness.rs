//! Configuration-only freshness at operation boundaries; native queries retain
//! admitted compiler authority and never parse live configuration themselves.
use super::super::config::host::ConfigLoadFailureIdentity;
use super::{
    Arc, LoadedLspConfig, Path, PathBuf, ProjectContextKey, RetainedConfiguration, ServerState, Url,
};
use vize_canon::ConfigurationObservations;

#[derive(Clone, Eq, PartialEq)]
pub(in crate::server::state) struct ConfigurationObservation {
    observations: ConfigurationObservations,
    before_inputs: ConfigurationObservations,
    vite_paths: Vec<PathBuf>,
    unverified: bool,
    retry_unobservable: bool,
    notified_error: Option<vize_l0::String>,
    notified_identity: Option<ConfigLoadFailureIdentity>,
    pub(super) rejected: bool,
}

impl ConfigurationObservation {
    pub(in crate::server::state) fn initial(
        raw: &LoadedLspConfig,
        loaded: &LoadedLspConfig,
        compiler: ConfigurationObservations,
        accepted: bool,
        before: Option<&ConfigurationObservations>,
        needs_confirmation: bool,
    ) -> Self {
        if !accepted {
            return Self::failed(raw, loaded, &compiler);
        }
        if let Some(before) = before {
            let mut record = Self::checked(raw, loaded, before, &compiler, None);
            record.unverified |= needs_confirmation;
            return record;
        }
        let mut observations = compiler;
        observations.extend(&ConfigurationObservations::capture(
            &loaded.watch_dependencies,
        ));
        Self {
            observations,
            before_inputs: ConfigurationObservations::default(),
            vite_paths: raw.watch_dependencies.clone(),
            // Public Vite bundling reports paths after evaluation, not consumed
            // bytes. Confirm these paths once at the next operation boundary.
            unverified: !raw.watch_dependencies.is_empty(),
            retry_unobservable: false,
            notified_error: None,
            notified_identity: None,
            rejected: false,
        }
    }

    pub(super) fn checked(
        raw: &LoadedLspConfig,
        applied: &LoadedLspConfig,
        before: &ConfigurationObservations,
        compiler: &ConfigurationObservations,
        post_read: Option<&mut super::config_reload::ConfigurationPostRead>,
    ) -> Self {
        let mut observations = before.clone();
        observations.extend(compiler);
        let phase = vize_canon::corsa_bridge::EditorPreparationPhase::start(
            "configuration_observation_membership",
            raw.watch_dependencies.len(),
        );
        let known = before
            .paths()
            .into_iter()
            .collect::<vize_l0::FxHashSet<_>>();
        let unverified = raw
            .watch_dependencies
            .iter()
            .any(|path| !known.contains(path));
        phase.finish();
        // This post-read observes future repair only. `unverified` explicitly
        // prevents it from being represented as an evaluated-byte receipt.
        observations.extend(&match post_read {
            Some(post_read) => post_read.capture(&applied.watch_dependencies),
            None => ConfigurationObservations::capture(&applied.watch_dependencies),
        });
        Self {
            observations,
            before_inputs: before.clone(),
            vite_paths: raw.watch_dependencies.clone(),
            unverified,
            retry_unobservable: false,
            notified_error: None,
            notified_identity: None,
            rejected: false,
        }
    }

    pub(super) fn failed(
        raw: &LoadedLspConfig,
        loaded: &LoadedLspConfig,
        observed: &ConfigurationObservations,
    ) -> Self {
        let needs_confirmation = observed.needs_confirmation();
        let mut paths = observed.paths();
        paths.extend(loaded.watch_dependencies.iter().cloned());
        paths.extend(loaded.project.dependencies.iter().cloned());
        Self {
            // A rejected attempt's end-state is only a suppression/repair
            // baseline. It never supplies compiler options or source authority.
            observations: ConfigurationObservations::capture(&paths),
            before_inputs: observed.clone(),
            vite_paths: raw.watch_dependencies.clone(),
            // A rejection caused by changed consumed inputs must confirm the
            // now-stable revision, even when the end-state no longer changes.
            unverified: needs_confirmation,
            retry_unobservable: !raw.watch_dependencies_complete,
            notified_error: None,
            notified_identity: None,
            rejected: true,
        }
    }

    /// Retry eligibility is not evidence that every observed input changed.
    pub(super) fn retry_required(&self) -> bool {
        self.unverified || self.retry_unobservable
    }

    pub(super) fn admits_initial_continuation(&self) -> bool {
        !self.rejected && !self.retry_unobservable
    }

    pub(super) fn changes(&self) -> Vec<PathBuf> {
        self.observations.changed_paths()
    }

    pub(super) fn input_observations(&self) -> &ConfigurationObservations {
        &self.observations
    }
    pub(super) fn before_observations(&self) -> ConfigurationObservations {
        self.before_inputs.clone()
    }
    pub(super) fn needs_confirmation(&self) -> bool {
        self.unverified
    }
    pub(super) fn stable_confirmation(&self) -> bool {
        self.unverified
            && !self.rejected
            && !self.retry_unobservable
            && self.observations.changed_paths().is_empty()
    }
    pub(super) fn require_confirmation(&mut self) {
        self.unverified = true;
    }

    pub(super) fn repeats_notified_failure(
        &self,
        error: &str,
        identity: Option<&ConfigLoadFailureIdentity>,
    ) -> bool {
        let same_failure = match (&self.notified_identity, identity) {
            (Some(previous), Some(current)) => previous == current,
            _ => self.notified_error.as_deref() == Some(error),
        };
        same_failure && self.observations.changed_paths().is_empty()
    }

    pub(super) fn mark_failure_notified(
        &mut self,
        error: &str,
        identity: Option<&ConfigLoadFailureIdentity>,
    ) {
        self.notified_error = Some(error.into());
        self.notified_identity = identity.cloned();
    }

    pub(super) fn paths(&self) -> Vec<PathBuf> {
        self.observations.paths()
    }
}

impl ServerState {
    pub(super) fn retains_configuration_authority(&self, next: &RetainedConfiguration) -> bool {
        let applied = self.project_contexts.applied_config.read();
        let compiler = self.project_contexts.compiler_configuration.read();
        applied
            .as_ref()
            .zip(compiler.as_ref())
            .is_some_and(|(applied, compiler)| next.same_authority(applied, compiler))
    }

    pub(super) fn configuration_authority_matches(
        &self,
        key: &ProjectContextKey,
        previous: Option<&Arc<super::registry::ProjectContext>>,
        next: &RetainedConfiguration,
    ) -> bool {
        previous.map_or_else(
            || {
                key.tsconfig.is_none()
                    && self.is_primary_boundary(&key.root)
                    && self.retains_configuration_authority(next)
            },
            |previous| previous.same_authority(next),
        )
    }

    pub(super) fn prepare_configuration_attempt(
        key: &ProjectContextKey,
        raw: &LoadedLspConfig,
        before: &ConfigurationObservations,
        dependencies: &[PathBuf],
        post_read: Option<&mut super::config_reload::ConfigurationPostRead>,
    ) -> (
        Result<RetainedConfiguration, vize_l0::String>,
        ConfigurationObservation,
    ) {
        let mut compiler_observations = ConfigurationObservations::default();
        let mut applied = raw.clone();
        let mut changed_before_admission = false;
        let snapshot = if let Some(error) = &raw.error {
            Err(vize_l0::String::from(error.as_str()))
        } else {
            Self::prepare_tsconfig_config_snapshot(&key.root, Some(key), &mut applied, &mut compiler_observations)
                .map(|compiler| RetainedConfiguration {
                    observation: ConfigurationObservation::checked(raw, &applied, before, &compiler_observations, post_read),
                    authored: raw.clone(), applied: applied.clone(), compiler,
                })
        }.and_then(|snapshot| {
            changed_before_admission = !before.changed_paths().is_empty();
            if !changed_before_admission { Ok(snapshot) } else {
                Err(vize_l0::cstr!("project configuration changed during evaluation; keeping the last good configuration"))
            }
        });
        compiler_observations.extend(before);
        let observation = match &snapshot {
            Ok(snapshot) => snapshot.observation.clone(),
            Err(_) => {
                compiler_observations.extend(&ConfigurationObservations::capture(dependencies));
                let mut failed =
                    ConfigurationObservation::failed(raw, &applied, &compiler_observations);
                if changed_before_admission {
                    failed.require_confirmation();
                }
                failed
            }
        };
        (snapshot, observation)
    }

    pub(super) fn configuration_records(
        &self,
    ) -> Vec<(ProjectContextKey, ConfigurationObservation)> {
        let mut records = self
            .project_contexts
            .contexts
            .lock()
            .iter()
            .filter_map(|(key, context)| {
                context
                    .configuration_observation()
                    .map(|record| (key.clone(), record))
            })
            .collect::<vize_l0::FxHashMap<_, _>>();
        if let Some(record) = self
            .project_contexts
            .configuration_observation
            .read()
            .clone()
            && let Some(root) = self
                .project_contexts
                .boundary
                .read()
                .clone()
                .or_else(|| self.get_workspace_root())
        {
            records.entry(root.into()).or_insert(record);
        }
        // Latest failed attempts supersede admission observations for polling,
        // while admitted dependencies remain separately watched below.
        records.extend(self.project_contexts.configuration_attempts.lock().clone());
        records.into_iter().collect()
    }

    pub(super) fn configuration_paths_for_key(&self, key: &ProjectContextKey) -> Vec<PathBuf> {
        self.configuration_records()
            .into_iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, record)| record.paths())
            .unwrap_or_default()
    }

    pub(super) fn configuration_vite_paths_for_root(&self, root: &Path) -> Vec<PathBuf> {
        let mut paths = self
            .configuration_records()
            .into_iter()
            .filter(|(key, _)| key.root == root)
            .flat_map(|(_, record)| record.vite_paths)
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        paths
    }

    #[cfg(test)]
    pub(super) fn configuration_watch_paths_for_root(&self, root: &Path) -> Vec<PathBuf> {
        let mut paths = self
            .configuration_records()
            .into_iter()
            .filter(|(key, _)| key.root == root)
            .flat_map(|(_, record)| record.paths())
            .collect::<Vec<_>>();
        if self.is_primary_boundary(root) {
            paths.extend(
                self.project_contexts
                    .config_dependencies
                    .read()
                    .iter()
                    .cloned(),
            );
        }
        for (key, context) in self.project_contexts.contexts.lock().iter() {
            if key.root == root {
                paths.extend(context.config_dependencies());
            }
        }
        paths.sort();
        paths.dedup();
        paths
    }

    /// Admitted and latest-attempt dependencies, solely for repair watching.
    pub(crate) fn configuration_watch_paths(&self) -> Vec<PathBuf> {
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(super::Weak::upgrade)
        {
            return owner.configuration_watch_paths();
        }
        // One current snapshot supplies every retained/failed observation. A
        // per-root call would clone all records again for each workspace owner.
        let records = self.configuration_records();
        let roots = records
            .iter()
            .map(|(key, _)| key.root.clone())
            .collect::<vize_l0::FxHashSet<_>>();
        let mut paths = records
            .into_iter()
            .flat_map(|(_, record)| record.paths())
            .collect::<Vec<_>>();
        if roots.iter().any(|root| self.is_primary_boundary(root)) {
            paths.extend(
                self.project_contexts
                    .config_dependencies
                    .read()
                    .iter()
                    .cloned(),
            );
        }
        for (key, context) in self.project_contexts.contexts.lock().iter() {
            if roots.contains(&key.root) {
                paths.extend(context.config_dependencies());
            }
        }
        paths.sort();
        paths.dedup();
        paths
    }

    pub(crate) fn document_configuration_root(&self, uri: &Url) -> Option<PathBuf> {
        let owner = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(super::Weak::upgrade);
        let state = owner.as_deref().unwrap_or(self);
        uri.to_file_path()
            .ok()
            .and_then(|path| state.document_context_root(&path))
    }

    pub(crate) async fn refresh_document_configuration_async(
        self: &Arc<Self>,
        uri: &Url,
    ) -> super::config_reload::ProjectConfigChanges {
        let owner = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(super::Weak::upgrade);
        let state = owner.as_ref().unwrap_or(self);
        let Some(root) = uri
            .to_file_path()
            .ok()
            .and_then(|path| state.document_context_root(&path))
        else {
            return Default::default();
        };
        let cached = state
            .project_contexts
            .routes
            .get(uri)
            .map(|route| route.value().clone());
        if let Some(previous) = cached
            && previous.root != root
        {
            return state
                .refresh_configuration_topology_async(uri, previous, root)
                .await;
        }
        let gates = state.config_reload_gates(std::slice::from_ref(&root));
        let Some(gate) = gates.first() else {
            return Default::default();
        };
        let admission = gate.clone().lock_owned().await;
        let checked_root = root.clone();
        let worker = state.clone();
        let outcome = Self::project_background(move || {
            let _admission = admission;
            let mut affected = Vec::new();
            let mut confirmations = Vec::new();
            let mut changes = Vec::new();
            for (key, record) in worker
                .configuration_records()
                .into_iter()
                .filter(|(key, _)| key.root == root)
            {
                let changed = record.changes();
                if record.retry_required() || !changed.is_empty() {
                    if record.stable_confirmation() {
                        confirmations.push((key.clone(), record.input_observations().clone()));
                    }
                    affected.push(key);
                    changes.extend(
                        changed
                            .into_iter()
                            .filter_map(|path| Url::from_file_path(path).ok()),
                    );
                }
            }
            changes.sort();
            changes.dedup();
            if affected.is_empty() || worker.project_routing_shutting_down() {
                return Default::default();
            }
            if affected.iter().any(|key| key.tsconfig.is_none()) {
                affected.extend(
                    worker
                        .project_contexts
                        .contexts
                        .lock()
                        .iter()
                        .filter(|(key, context)| {
                            key.root == root
                                && key.tsconfig.is_some()
                                && context.initialized_state().is_none()
                        })
                        .map(|(key, _)| key.clone()),
                );
            }
            affected.sort_by(|a, b| (&a.root, &a.tsconfig).cmp(&(&b.root, &b.tsconfig)));
            affected.dedup();
            let mut scope = super::watched::ConfigEventScope::for_owners(affected);
            scope.confirmations = confirmations;
            // One checked attempt, with the same per-root admission/CAS route
            // as file watching. No synthetic protocol notification or retry loop.
            worker.apply_project_config_paths(&changes, scope)
        })
        .await;
        match outcome {
            Some(mut result) => {
                result.checked_roots.push(checked_root);
                result
            }
            None => {
                if state.project_routing_shutting_down() {
                    return Default::default();
                }
                super::config_reload::ProjectConfigChanges {
                    errors: vec![super::config_reload::worker_unavailable_error()],
                    ..Default::default()
                }
            }
        }
    }
}
