//! Prepare configuration once per Vite root; install only valid full-key owners.
use super::config_reload::{ProjectConfigChanges, known_config_events, reload_error};
use super::{Arc, Ordering, ServerState, is_config_marker, paths::physical_path, registry};
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::{FileEvent, Url};
use vize_l0::{FxHashMap, FxHashSet};

pub(super) use super::config_reload::ConfigEventScope;

impl ServerState {
    pub(super) fn project_config_event_scope(&self, changes: &[FileEvent]) -> ConfigEventScope {
        let paths = changes
            .iter()
            .map(|event| event.uri.clone())
            .collect::<Vec<_>>();
        self.project_config_path_scope(&paths)
    }

    pub(super) fn project_config_path_scope(&self, changes: &[Url]) -> ConfigEventScope {
        let empty = || ConfigEventScope {
            affected: Vec::new(),
            confirmations: Vec::new(),
            marker_paths: Vec::new(),
        };
        if self.project_contexts.owner.read().is_some() || self.project_routing_shutting_down() {
            return empty();
        }
        let mut roots = self.workspace_root_paths();
        if let Some(boundary) = self.project_contexts.boundary.read().clone()
            && !roots.contains(&boundary)
        {
            roots.push(boundary);
        }
        let physical_roots = roots
            .iter()
            .map(|root| physical_path(root))
            .collect::<Vec<_>>();
        let changed = changes
            .iter()
            .filter_map(|uri| uri.to_file_path().ok())
            .map(|path| physical_path(&path))
            .collect::<Vec<_>>();
        let marker_paths = changes
            .iter()
            .filter_map(|uri| uri.to_file_path().ok())
            .filter(|path| !excluded_config_path(path, &roots))
            .map(|path| physical_path(&path))
            .filter(|path| is_config_marker(path) && !excluded_config_path(path, &physical_roots))
            .collect::<Vec<_>>();
        let depends = |dependencies: &[PathBuf]| {
            dependencies
                .iter()
                .any(|dependency| changed.contains(&physical_path(dependency)))
        };
        let observations = self
            .configuration_records()
            .into_iter()
            .collect::<FxHashMap<_, _>>();
        let mut primary_dependencies = self.project_contexts.config_dependencies.read().clone();
        if let Some(root) = self
            .project_contexts
            .boundary
            .read()
            .clone()
            .or_else(|| self.get_workspace_root())
            && let Some(record) = observations.get(&root.into())
        {
            primary_dependencies.extend(record.paths());
        }
        let contexts = self
            .project_contexts
            .contexts
            .lock()
            .iter()
            .map(|(key, context)| (key.clone(), context.clone()))
            .collect::<Vec<_>>();
        let mut tracked = primary_dependencies
            .iter()
            .map(|path| physical_path(path))
            .collect::<Vec<_>>();
        for (_, context) in &contexts {
            tracked.extend(
                context
                    .config_dependencies()
                    .iter()
                    .map(|path| physical_path(path)),
            );
        }
        let untracked = marker_paths
            .iter()
            .filter(|path| !tracked.contains(path))
            .map(|path| {
                (
                    path.clone(),
                    super::paths::document_context_root(path, &roots),
                )
            })
            .collect::<Vec<_>>();
        // A new boundary for an open buffer checks its own policy first. Its
        // ancestor's unchanged policy must survive rejection in another worker.
        let new_boundaries = untracked
            .iter()
            .filter_map(|(_, root)| root.as_ref())
            .filter(|root| {
                self.documents.iter().any(|document| {
                    document
                        .key()
                        .to_file_path()
                        .is_ok_and(|path| physical_path(&path).starts_with(physical_path(root)))
                })
            })
            .cloned()
            .collect::<FxHashSet<_>>();
        let mut affected = contexts
            .iter()
            .filter(|(key, context)| {
                depends(&context.config_dependencies())
                    || observations
                        .get(key)
                        .is_some_and(|record| depends(&record.paths()))
                    || untracked.iter().any(|(path, nearest)| {
                        path.starts_with(physical_path(&key.root))
                            && nearest.as_ref().is_none_or(|root| {
                                root == &key.root || !new_boundaries.contains(root)
                            })
                    })
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        if let Some(root) = self
            .project_contexts
            .boundary
            .read()
            .clone()
            .or_else(|| self.get_workspace_root())
            && (depends(&primary_dependencies)
                || marker_paths
                    .iter()
                    .any(|path| path.parent() == Some(physical_path(&root).as_path())))
        {
            affected.push(root.into());
        }
        let base_roots = affected
            .iter()
            .filter(|key| key.tsconfig.is_none())
            .map(|key| key.root.clone())
            .collect::<Vec<_>>();
        affected.extend(
            contexts
                .iter()
                .filter(|(key, context)| {
                    key.tsconfig.is_some()
                        && base_roots.contains(&key.root)
                        && context.initialized_state().is_none()
                })
                .map(|(key, _)| key.clone()),
        );
        drop(contexts);
        if changed.iter().any(|path| {
            !excluded_config_path(path, &physical_roots)
                && physical_roots.iter().any(|root| path.starts_with(root))
        }) {
            affected.extend(
                self.project_contexts
                    .pending_config_reloads
                    .lock()
                    .iter()
                    .cloned(),
            );
        }
        for root in new_boundaries {
            if !affected.iter().any(|key| {
                key.tsconfig.is_none() && physical_path(&key.root) == physical_path(&root)
            }) {
                affected.push(root.into());
            }
        }
        affected.sort_by(|a, b| (&a.root, &a.tsconfig).cmp(&(&b.root, &b.tsconfig)));
        affected.dedup();
        ConfigEventScope {
            affected,
            confirmations: Vec::new(),
            marker_paths,
        }
    }

    pub(super) fn apply_project_config_events(
        self: &Arc<Self>,
        changes: &[FileEvent],
        scope: ConfigEventScope,
    ) -> ProjectConfigChanges {
        let paths = changes
            .iter()
            .map(|event| event.uri.clone())
            .collect::<Vec<_>>();
        self.apply_project_config_paths(&paths, scope)
    }

    pub(super) fn apply_project_config_paths(
        self: &Arc<Self>,
        changes: &[Url],
        scope: ConfigEventScope,
    ) -> ProjectConfigChanges {
        let mut result = ProjectConfigChanges::default();
        if scope.affected.is_empty() || self.project_routing_shutting_down() {
            return result;
        }
        let contexts = self.project_contexts.contexts.lock();
        if self.project_routing_shutting_down() {
            return result;
        }
        self.project_contexts
            .pending_config_reloads
            .lock()
            .extend(scope.affected.iter().cloned());
        let previous = scope
            .affected
            .iter()
            .map(|key| (key.clone(), contexts.get(key).cloned()))
            .collect::<Vec<_>>();
        drop(contexts);
        // Canonical JS evaluation is shared by every referenced owner in one
        // Vite root. Only the applied TypeScript snapshot varies by full key.
        let mut authored = FxHashMap::default();
        let mut prepared = Vec::new();
        for (key, previous) in previous {
            let mut dependencies = previous
                .as_ref()
                .map(|context| context.config_dependencies())
                .unwrap_or_else(|| self.project_contexts.config_dependencies.read().clone());
            dependencies.extend(self.configuration_paths_for_key(&key));
            let markers = if key.tsconfig.is_none() {
                scope.marker_paths.as_slice()
            } else {
                &[]
            };
            let changed = known_config_events(&key.root, &dependencies, markers, changes);
            result.configuration_events.extend(changed.clone());
            let (raw, before, post_read) = authored.entry(key.root.clone()).or_insert_with(|| {
                let mut before = vize_canon::ConfigurationObservations::capture(
                    &self.configuration_vite_paths_for_root(&key.root),
                );
                let loaded = super::load_observed_lsp_config_snapshot(
                    Some(&key.root),
                    &self.configuration_evaluation_session(),
                );
                before.extend(&loaded.before);
                let post_read = super::config_reload::ConfigurationPostRead::new(
                    &loaded.loaded.watch_dependencies,
                );
                (loaded.loaded, before, post_read)
            });
            let (snapshot, observation) = Self::prepare_configuration_attempt(
                &key,
                raw,
                before,
                &dependencies,
                Some(post_read),
            );
            let repeated_failure = snapshot.as_ref().err().is_some_and(|error| {
                self.project_contexts
                    .configuration_attempts
                    .lock()
                    .get(&key)
                    .is_some_and(|record| {
                        record.repeats_notified_failure(error, raw.failure_identity.as_ref())
                    })
            });
            if let Ok(snapshot) = &snapshot {
                result.configuration_events.extend(known_config_events(
                    &key.root,
                    &snapshot.applied.project.dependencies,
                    &[],
                    changes,
                ));
            }
            let confirmation_only = scope.confirmations.iter().any(|(candidate, original)| {
                candidate == &key && original.changed_paths().is_empty()
            }) && snapshot.as_ref().is_ok_and(|next| {
                self.configuration_authority_matches(&key, previous.as_ref(), next)
            });
            prepared.push((
                key,
                previous,
                snapshot,
                changed,
                observation,
                (repeated_failure, raw.failure_identity.clone()),
                confirmation_only,
            ));
        }
        result.checked_roots.extend(authored.keys().cloned());
        #[cfg(test)]
        super::lock_order_tests::before_config_install();
        let mut contexts = self.project_contexts.contexts.lock();
        if self.project_routing_shutting_down() {
            return ProjectConfigChanges::default();
        }
        let mut routing_change = None;
        let mut accepted_primary = false;
        let mut retired = Vec::new();
        let mut published_keys = Vec::new();
        let mut accepted_roots = FxHashSet::default();
        let mut failed_roots = FxHashSet::default();
        for (
            key,
            previous,
            snapshot,
            changed,
            mut observation,
            (repeated_failure, failure_identity),
            confirmation_only,
        ) in prepared
        {
            let unchanged = match (&previous, contexts.get(&key)) {
                (Some(before), Some(current)) => Arc::ptr_eq(before, current),
                (None, None) => true,
                _ => false,
            };
            if !unchanged {
                continue;
            }
            if previous.is_none() && !self.admits_new_config_root(&key.root) {
                self.project_contexts
                    .pending_config_reloads
                    .lock()
                    .remove(&key);
                continue;
            }
            if let Err(error) = &snapshot {
                observation.mark_failure_notified(error, failure_identity.as_ref());
            }
            self.project_contexts
                .configuration_attempts
                .lock()
                .insert(key.clone(), observation);
            match snapshot {
                Ok(snapshot) => {
                    if confirmation_only
                        && scope.confirmations.iter().any(|(candidate, original)| {
                            candidate == &key && original.changed_paths().is_empty()
                        })
                    {
                        self.project_contexts
                            .pending_config_reloads
                            .lock()
                            .remove(&key);
                        if let Some(previous) = &previous {
                            previous.confirm_observation(&snapshot);
                        } else {
                            *self.project_contexts.configuration_observation.write() =
                                Some(snapshot.observation.clone());
                        }
                        continue;
                    }
                    published_keys.push(key.clone());
                    routing_change.get_or_insert_with(|| self.project_routing_change());
                    self.project_contexts
                        .pending_config_reloads
                        .lock()
                        .remove(&key);
                    let replacement = Arc::new(registry::ProjectContext::with_prepared(
                        key.clone(),
                        snapshot,
                    ));
                    if let Some(previous) = contexts.insert(key.clone(), replacement) {
                        previous.mark_retired();
                        retired.push(previous);
                    }
                    accepted_roots.insert(key.root.clone());
                    if key.tsconfig.is_none() && self.is_primary_boundary(&key.root) {
                        self.project_contexts.retired.store(true, Ordering::Release);
                        accepted_primary = true;
                    }
                }
                Err(error) => {
                    published_keys.push(key.clone());
                    tracing::warn!(project=%key.root.display(),tsconfig=?key.tsconfig,%error,"Keeping the last good project configuration");
                    self.project_contexts
                        .pending_config_reloads
                        .lock()
                        .insert(key.clone());
                    if let Some(previous) = &previous {
                        previous.record_reload_failure();
                    } else if key.tsconfig.is_none() && self.is_primary_boundary(&key.root) {
                        self.project_contexts
                            .config_reload_failed
                            .store(true, Ordering::Release);
                    }
                    failed_roots.insert(key.root.clone());
                    if !repeated_failure {
                        result.errors.push(reload_error(&key.root, &changed));
                    }
                    result.rejected.push((key, changed));
                }
            }
        }
        drop(contexts);
        // Applying a shell policy is not proof that a malformed referenced
        // leaf has a valid new membership graph. Preserve its routed dirty
        // buffers and existing native owner until topology becomes valid.
        let topology_roots = accepted_roots
            .difference(&failed_roots)
            .cloned()
            .collect::<Vec<_>>();
        let affected_documents = self
            .documents
            .iter()
            .filter_map(|document| {
                document
                    .key()
                    .to_file_path()
                    .ok()
                    .filter(|path| {
                        let route = self.project_contexts.routes.get(document.key());
                        published_keys.iter().any(|key| {
                            if route.as_ref().is_some_and(|route| route.value() == key) {
                                return true;
                            }
                            // New package markers may move a buffer out of its
                            // previous boundary. A shared directory alone must
                            // not republish a different referenced owner.
                            key.tsconfig.is_none()
                                && route.as_ref().is_none_or(|route| route.root != key.root)
                                && physical_path(path).starts_with(physical_path(&key.root))
                        })
                    })
                    .map(|_| (document.key().clone(), document.value().version))
            })
            .collect::<Vec<_>>();
        #[cfg(test)]
        if routing_change.is_some() {
            super::lock_order_tests::before_route_changes();
        }
        let moved_primary_documents = affected_documents
            .iter()
            .filter(|(uri, _)| {
                self.project_contexts
                    .routes
                    .get(uri)
                    .is_some_and(|previous| {
                        previous.tsconfig.is_none()
                            && self.is_primary_boundary(&previous.root)
                            && topology_roots.contains(&previous.root)
                            && uri
                                .to_file_path()
                                .ok()
                                .and_then(|path| self.document_context_root(&path))
                                .is_some_and(|current| current != previous.root)
                    })
            })
            .map(|(uri, _)| uri.clone())
            .collect::<Vec<_>>();
        if !topology_roots.is_empty() {
            if failed_roots.is_empty() {
                *self.project_contexts.reference_ownership.lock() = Default::default();
            }
            let transferred =
                self.retarget_checked_topology_documents(&topology_roots, changes, &mut result);
            self.project_contexts.routes.retain(|uri, key| {
                !topology_roots.contains(&key.root) || transferred.contains(uri)
            });
        }
        if accepted_primary {
            self.retire_project_owner();
        }
        for previous in retired {
            previous.retire();
        }
        for uri in moved_primary_documents {
            self.remove_virtual_docs(&uri);
        }
        result.errors.sort_by(|a, b| a.message.cmp(&b.message));
        result
            .errors
            .dedup_by(|a, b| a.typ == b.typ && a.message == b.message);
        result.documents.extend(affected_documents);
        result.documents.sort();
        result.documents.dedup();
        result
    }
}

pub(super) fn excluded_config_path(path: &Path, roots: &[PathBuf]) -> bool {
    let relative = roots
        .iter()
        .filter_map(|root| {
            path.strip_prefix(root)
                .ok()
                .map(|relative| (root, relative))
        })
        .max_by_key(|(root, _)| root.components().count())
        .map(|(_, relative)| relative);
    relative.is_some_and(|relative| relative.components().any(|part| matches!(part,std::path::Component::Normal(name) if matches!(name.to_str(),Some("node_modules"|".git"|"target"|"dist"|"build")))))
}
