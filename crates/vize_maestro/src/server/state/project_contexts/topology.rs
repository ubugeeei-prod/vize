//! A newly discovered boundary is checked before replacing a retained route.
use super::config_reload::{ProjectConfigChanges, reload_error, worker_unavailable_error};
use super::watched::ConfigEventScope;
use super::{Arc, Path, PathBuf, ProjectContextKey, ServerState, Url, registry};
use vize_canon::ConfigurationObservations;

impl ServerState {
    pub(super) async fn refresh_configuration_topology_async(
        self: &Arc<Self>,
        uri: &Url,
        previous: ProjectContextKey,
        target: PathBuf,
    ) -> ProjectConfigChanges {
        let mut roots = vec![previous.root.clone(), target.clone()];
        roots.sort();
        roots.dedup();
        let gates = self.config_reload_gates(&roots);
        let mut admissions = Vec::new();
        for gate in gates {
            admissions.push(gate.lock_owned().await);
        }
        let worker = self.clone();
        let uri = uri.clone();
        Self::project_background(move || {
            let _admissions = admissions;
            worker.reconcile_configuration_topology(&uri, &previous, &target)
        })
        .await
        .unwrap_or_else(|| {
            if self.project_routing_shutting_down() {
                return Default::default();
            }
            ProjectConfigChanges {
                errors: vec![worker_unavailable_error()],
                ..Default::default()
            }
        })
    }

    fn topology_owner(self: &Arc<Self>, key: &ProjectContextKey) -> Option<Arc<Self>> {
        let context = self.project_contexts.contexts.lock().get(key).cloned();
        if let Some(context) = context {
            return context.initialize(self);
        }
        (key.tsconfig.is_none()
            && self.is_primary_boundary(&key.root)
            && self
                .project_contexts
                .compiler_configuration
                .read()
                .is_some()
            && !self
                .project_contexts
                .config_reload_failed
                .load(super::Ordering::Acquire))
        .then(|| self.clone())
    }

    fn topology_record(
        &self,
        key: &ProjectContextKey,
    ) -> Option<super::freshness::ConfigurationObservation> {
        self.configuration_records()
            .into_iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, record)| record)
    }

    fn reconcile_configuration_topology(
        self: &Arc<Self>,
        uri: &Url,
        previous: &ProjectContextKey,
        target: &Path,
    ) -> ProjectConfigChanges {
        if self.project_routing_shutting_down()
            || self
                .project_contexts
                .routes
                .get(uri)
                .is_none_or(|route| route.value() != previous)
            || self.document_configuration_root(uri).as_deref() != Some(target)
        {
            return Default::default();
        }
        let target_key = ProjectContextKey::from(target.to_path_buf());
        let mut result = ProjectConfigChanges::default();
        let record = self.topology_record(&target_key);
        let changed = record.as_ref().map(|record| record.changes());
        if record
            .as_ref()
            .is_some_and(|record| record.rejected && !record.retry_required())
            && changed.as_ref().is_some_and(Vec::is_empty)
        {
            result.checked_roots.push(target.to_path_buf());
            return result;
        }
        let mut paths = self
            .configuration_records()
            .into_iter()
            .filter(|(key, _)| key.root == previous.root)
            .flat_map(|(_, record)| record.changes())
            .collect::<Vec<_>>();
        if let Some(changed) = changed.as_ref() {
            paths.extend(changed.iter().cloned());
        }
        // These are actual present discovery markers, not guessed failed imports.
        let markers = std::fs::read_dir(target)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| super::is_config_marker(path))
            .collect::<Vec<_>>();
        paths.extend(markers.iter().cloned());
        paths.sort();
        paths.dedup();
        let changes = paths
            .iter()
            .filter_map(|path| Url::from_file_path(path).ok())
            .collect::<Vec<_>>();
        if record.as_ref().is_none_or(|record| record.retry_required())
            || changed.as_ref().is_some_and(|paths| !paths.is_empty())
            || self.topology_owner(&target_key).is_none()
        {
            result = self.apply_project_config_paths(
                &changes,
                ConfigEventScope::for_topology(vec![target_key.clone()], paths),
            );
            if result.rejected.iter().any(|(key, _)| key == &target_key) {
                return result;
            }
        } else {
            result.checked_roots.push(target.to_path_buf());
        }
        let Some(target_owner) = self.topology_owner(&target_key) else {
            return result;
        };
        if target_owner
            .project_contexts
            .config_reload_failed
            .load(super::Ordering::Acquire)
        {
            return result;
        }
        self.transfer_topology_documents(previous, target, &target_owner, &changes, &mut result);
        result
    }

    /// A watched marker has already admitted these roots in this operation.
    /// Transfer cached routes through those snapshots without evaluating again.
    pub(super) fn retarget_checked_topology_documents(
        self: &Arc<Self>,
        roots: &[PathBuf],
        changes: &[Url],
        result: &mut ProjectConfigChanges,
    ) -> Vec<Url> {
        let mut transitions = self
            .documents
            .uris()
            .into_iter()
            .filter_map(|uri| {
                let previous = self.project_contexts.routes.get(&uri)?.value().clone();
                let target = self.document_configuration_root(&uri)?;
                (target != previous.root && roots.contains(&target)).then_some((previous, target))
            })
            .collect::<Vec<_>>();
        transitions.sort_by(|(a, target_a), (b, target_b)| {
            (&a.root, &a.tsconfig, target_a).cmp(&(&b.root, &b.tsconfig, target_b))
        });
        transitions.dedup();
        let mut transferred = Vec::new();
        for (previous, target) in transitions {
            let Some(owner) = self.topology_owner(&target.clone().into()) else {
                continue;
            };
            transferred.extend(
                self.transfer_topology_documents(&previous, &target, &owner, changes, result),
            );
        }
        transferred
    }

    fn transfer_topology_documents(
        self: &Arc<Self>,
        previous: &ProjectContextKey,
        target: &Path,
        target_owner: &Arc<Self>,
        changes: &[Url],
        result: &mut ProjectConfigChanges,
    ) -> Vec<Url> {
        let target_key = ProjectContextKey::from(target.to_path_buf());
        let moved = self
            .documents
            .uris()
            .into_iter()
            .filter(|document| {
                self.project_contexts
                    .routes
                    .get(document)
                    .is_some_and(|route| route.value() == previous)
                    && self.document_configuration_root(document).as_deref() == Some(target)
            })
            .collect::<Vec<_>>();
        let mut admitted = Vec::new();
        for document in moved {
            let key = self
                .document_reference_key(&document, target_owner)
                .unwrap_or_else(|| target_key.clone());
            if key == target_key
                || self.admit_topology_reference(&key, target_owner, changes, result)
            {
                admitted.push((document, key));
            }
        }
        if admitted.is_empty() {
            return Vec::new();
        }
        let _routing_change = self.project_routing_change();
        let mut transferred = Vec::new();
        for (document, key) in admitted {
            if self.document_configuration_root(&document).as_deref() != Some(target) {
                continue;
            }
            let mut route = match self.project_contexts.routes.get_mut(&document) {
                Some(route) if route.value() == previous => route,
                _ => continue,
            };
            *route = key.clone();
            drop(route);
            let old = self
                .project_contexts
                .contexts
                .lock()
                .get(previous)
                .and_then(|context| context.initialized_state());
            if let Some(old) = old {
                old.retire_document_project_membership(&document);
            } else if previous.tsconfig.is_none() && self.is_primary_boundary(&previous.root) {
                self.retire_document_project_membership(&document);
            }
            if let Some(owner) = self.topology_owner(&key)
                && let Some(source) = self.documents.text(&document)
            {
                owner.update_virtual_docs(&document, &source);
            }
            if let Some(version) = self.documents.version(&document) {
                result.documents.push((document.clone(), version));
                transferred.push(document);
            }
        }
        self.retire_unrouted_topology_owner(previous);
        result.documents.sort();
        result.documents.dedup();
        transferred
    }

    fn admit_topology_reference(
        self: &Arc<Self>,
        key: &ProjectContextKey,
        base: &Arc<Self>,
        changes: &[Url],
        result: &mut ProjectConfigChanges,
    ) -> bool {
        if let Some(record) = self.topology_record(key)
            && !record.retry_required()
            && record.changes().is_empty()
        {
            return !record.rejected && self.topology_owner(key).is_some();
        }
        let Some(raw) = base.project_contexts.loaded_config.read().clone() else {
            return false;
        };
        let previous = self.project_contexts.contexts.lock().get(key).cloned();
        let dependencies = previous
            .as_ref()
            .map(|context| context.config_dependencies())
            .unwrap_or_default();
        // Reuse the same admitted Vite value. Its newly reported paths remain
        // explicitly unverified; only Canon reads are captured by this phase.
        let base_key = ProjectContextKey::from(key.root.clone());
        let base_record = self.topology_record(&base_key);
        let empty = ConfigurationObservations::default();
        let before = base_record
            .as_ref()
            .map(|record| record.input_observations())
            .unwrap_or(&empty);
        let (mut snapshot, mut observation) =
            Self::prepare_configuration_attempt(key, &raw, before, &dependencies, None);
        if base_record
            .as_ref()
            .is_none_or(|record| record.needs_confirmation())
        {
            observation.require_confirmation();
            if let Ok(snapshot) = &mut snapshot {
                snapshot.observation.require_confirmation();
            }
        }
        let mut contexts = self.project_contexts.contexts.lock();
        let unchanged = match (&previous, contexts.get(key)) {
            (Some(before), Some(current)) => Arc::ptr_eq(before, current),
            (None, None) => true,
            _ => false,
        };
        if !unchanged {
            return false;
        }
        match snapshot {
            Ok(snapshot) => {
                let _routing_change = self.project_routing_change();
                let replacement = Arc::new(registry::ProjectContext::with_prepared(
                    key.clone(),
                    snapshot,
                ));
                let retired = contexts.insert(key.clone(), replacement);
                if let Some(retired) = &retired {
                    retired.mark_retired();
                }
                self.project_contexts
                    .configuration_attempts
                    .lock()
                    .insert(key.clone(), observation);
                self.project_contexts
                    .pending_config_reloads
                    .lock()
                    .remove(key);
                drop(contexts);
                if let Some(retired) = retired {
                    retired.retire();
                }
                true
            }
            Err(error) => {
                let repeated = self
                    .project_contexts
                    .configuration_attempts
                    .lock()
                    .get(key)
                    .is_some_and(|record| {
                        record.repeats_notified_failure(&error, raw.failure_identity.as_ref())
                    });
                observation.mark_failure_notified(&error, raw.failure_identity.as_ref());
                self.project_contexts
                    .configuration_attempts
                    .lock()
                    .insert(key.clone(), observation);
                self.project_contexts
                    .pending_config_reloads
                    .lock()
                    .insert(key.clone());
                if !repeated {
                    result.errors.push(reload_error(&key.root, changes));
                }
                result.rejected.push((key.clone(), changes.to_vec()));
                false
            }
        }
    }

    fn retire_unrouted_topology_owner(&self, key: &ProjectContextKey) {
        self.project_contexts
            .routes
            .retain(|uri, route| route != key || self.documents.contains(uri));
        if self
            .project_contexts
            .routes
            .iter()
            .any(|route| route.value() == key)
        {
            return;
        }
        if key.tsconfig.is_none() && self.is_primary_boundary(&key.root) {
            return;
        }
        let retired = self.project_contexts.contexts.lock().remove(key);
        if let Some(retired) = retired {
            retired.retire();
        }
        if !self.is_primary_boundary(&key.root) || key.tsconfig.is_some() {
            self.project_contexts
                .configuration_attempts
                .lock()
                .remove(key);
            self.project_contexts
                .pending_config_reloads
                .lock()
                .remove(key);
        }
    }
}
