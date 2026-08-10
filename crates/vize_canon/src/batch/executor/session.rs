//! Corsa project-session execution, including persistent incremental snapshots.

use std::{
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use crate::{corsa_client::CorsaProjectClient, file_uri::path_to_file_uri};
use vize_carton::{FxHashMap, String, hash::hash_bytes, profile};

use super::{
    CorsaExecutor, FallbackStep, MaterializeLock, TypeCheckResult, VirtualProject, check_with_cli,
    map_corsa_error, should_fallback_to_cli, warn_fallback,
};
use crate::batch::error::CorsaResult;
use crate::batch::executor::diagnostics::map_batch_diagnostics;
use crate::batch::type_checker::IncrementalCheckMetrics;
use crate::batch::virtual_project::{
    AUTO_IMPORT_STUBS_FILE, SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE,
};

mod diagnostic_paths;

use diagnostic_paths::{extend_diagnostic_path_uris, is_authored_diagnostic_input};

#[derive(Default)]
pub(super) struct IncrementalSessionState {
    session: Option<IncrementalSession>,
    pub(super) metrics: IncrementalCheckMetrics,
}

struct IncrementalSession {
    client: CorsaProjectClient,
    snapshot: MaterializedSnapshot,
}

#[derive(Default)]
struct MaterializedSnapshot {
    revisions: FxHashMap<PathBuf, u64>,
    uris: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct MaterializedDelta {
    changed: Vec<PathBuf>,
    created: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
}

impl CorsaExecutor {
    pub(super) fn check_with_project_session(
        &self,
        project: &VirtualProject,
    ) -> CorsaResult<TypeCheckResult> {
        let corsa_path = self.corsa_path.to_string_lossy();
        let mut client = match profile!(
            "canon.corsa.session",
            CorsaProjectClient::new_for_workspace(
                Some(corsa_path.as_ref()),
                project.virtual_root()
            )
        ) {
            Ok(client) => client,
            Err(error) if should_fallback_to_cli(&error) => {
                warn_fallback(
                    FallbackStep::SessionToCli,
                    &map_corsa_error(error.as_str().into()),
                );
                return profile!(
                    "canon.corsa.cli_fallback",
                    check_with_cli(&self.corsa_path, project)
                );
            }
            Err(error) => return Err(map_corsa_error(error)),
        };
        let mut uris = profile!(
            "canon.corsa.collect_uris",
            collect_virtual_file_uris(project.virtual_root())
        )?;
        extend_diagnostic_path_uris(project, &mut uris);
        check_session_client(&mut client, project, &uris)
    }

    pub(crate) fn check_incremental_session(
        &self,
        project: &VirtualProject,
        servers: Option<usize>,
    ) -> CorsaResult<TypeCheckResult> {
        let session_result = {
            let _materialize_lock = MaterializeLock::acquire(project.virtual_root())?;
            profile!(
                "canon.executor.materialize_incremental",
                project.materialize()
            )?;
            let mut snapshot = profile!(
                "canon.corsa.incremental.snapshot",
                MaterializedSnapshot::capture(project.virtual_root())
            )?;
            snapshot.extend_diagnostic_paths(project)?;
            let mut state = self
                .incremental_session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let result = state.check(&self.corsa_path, project, snapshot);
            if result.is_err() {
                state.session = None;
                state.metrics.session_to_cli_fallbacks += 1;
                state.metrics.last_session_to_cli_fallback = true;
            }
            result
        };

        match session_result {
            Ok(result) => Ok(result),
            Err(error) => {
                warn_fallback(FallbackStep::SessionToCli, &error);
                self.check_with_servers(project, servers)
            }
        }
    }
}

impl IncrementalSessionState {
    fn check(
        &mut self,
        corsa_path: &Path,
        project: &VirtualProject,
        snapshot: MaterializedSnapshot,
    ) -> CorsaResult<TypeCheckResult> {
        self.metrics.checks += 1;
        self.metrics.last_requested_files = snapshot.uris.len();
        self.metrics.last_session_started = false;
        self.metrics.last_session_reused = false;
        self.metrics.last_session_refreshed = false;
        self.metrics.last_session_to_cli_fallback = false;
        self.metrics.last_changed_files = 0;
        self.metrics.last_created_files = 0;
        self.metrics.last_deleted_files = 0;

        if let Some(session) = &mut self.session {
            self.metrics.last_session_reused = true;
            let delta = snapshot.diff(&session.snapshot);
            self.metrics.last_changed_files = delta.changed.len();
            self.metrics.last_created_files = delta.created.len();
            self.metrics.last_deleted_files = delta.deleted.len();
            let refreshed = !delta.is_empty();
            self.metrics.last_session_refreshed = refreshed;
            if refreshed {
                profile!(
                    "canon.corsa.incremental.refresh",
                    session.client.refresh_materialized_files(
                        &delta.changed,
                        &delta.created,
                        &delta.deleted
                    )
                )
                .map_err(map_corsa_error)?;
            }
            session.snapshot = snapshot;
            self.metrics.session_reuses += 1;
            self.metrics.session_refreshes += usize::from(refreshed);
        } else {
            let corsa_path = corsa_path.to_string_lossy();
            let client = profile!(
                "canon.corsa.incremental.start",
                CorsaProjectClient::new_for_workspace(
                    Some(corsa_path.as_ref()),
                    project.virtual_root()
                )
            )
            .map_err(map_corsa_error)?;
            self.session = Some(IncrementalSession { client, snapshot });
            self.metrics.session_starts += 1;
            self.metrics.last_session_started = true;
        }

        let session = self.session.as_mut().expect("session initialized above");
        check_session_client(&mut session.client, project, &session.snapshot.uris)
    }
}

impl MaterializedSnapshot {
    fn capture(virtual_root: &Path) -> CorsaResult<Self> {
        let mut snapshot = Self::default();
        for entry in walkdir::WalkDir::new(virtual_root) {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type().is_symlink() {
                let target = std::fs::read_link(path)?;
                let resolves_to_file = path.is_file();
                let revision = if resolves_to_file {
                    hash_bytes(&std::fs::read(path)?)
                } else {
                    hash_path(&target)
                };
                snapshot.revisions.insert(path.to_path_buf(), revision);
                if resolves_to_file
                    && is_diagnostic_input(path)
                    && !is_under_virtual_node_modules(virtual_root, path)
                    && !is_internal_virtual_project_stub(path)
                {
                    snapshot.uris.push(path_to_file_uri(path));
                }
                continue;
            }
            if !entry.file_type().is_file() {
                continue;
            }

            let content = std::fs::read(path)?;
            snapshot
                .revisions
                .insert(path.to_path_buf(), hash_bytes(&content));
            if is_diagnostic_input(path)
                && !is_under_virtual_node_modules(virtual_root, path)
                && !is_internal_virtual_project_stub(path)
            {
                snapshot.uris.push(path_to_file_uri(path));
            }
        }
        snapshot.uris.sort();
        Ok(snapshot)
    }

    fn diff(&self, previous: &Self) -> MaterializedDelta {
        let mut delta = MaterializedDelta::default();
        for (path, revision) in &self.revisions {
            match previous.revisions.get(path) {
                None => delta.created.push(path.clone()),
                Some(previous) if previous != revision => delta.changed.push(path.clone()),
                Some(_) => {}
            }
        }
        for path in previous.revisions.keys() {
            if !self.revisions.contains_key(path) {
                delta.deleted.push(path.clone());
            }
        }
        delta.sort();
        delta
    }

    fn extend_diagnostic_paths(&mut self, project: &VirtualProject) -> CorsaResult<()> {
        for path in project.diagnostic_paths_sorted() {
            if !path.is_file() || !is_authored_diagnostic_input(&path) {
                continue;
            }
            let content = std::fs::read(&path)?;
            self.revisions.insert(path.clone(), hash_bytes(&content));
            self.uris.push(path_to_file_uri(&path));
        }
        self.uris.sort();
        self.uris.dedup();
        Ok(())
    }
}

impl MaterializedDelta {
    fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.created.is_empty() && self.deleted.is_empty()
    }

    fn sort(&mut self) {
        self.changed.sort();
        self.created.sort();
        self.deleted.sort();
    }
}

fn hash_path(path: &Path) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

fn check_session_client(
    client: &mut CorsaProjectClient,
    project: &VirtualProject,
    uris: &[String],
) -> CorsaResult<TypeCheckResult> {
    let raw_diagnostics = profile!(
        "canon.corsa.diagnostics",
        client
            .request_diagnostics_batch(uris)
            .map_err(map_corsa_error)
    )?;
    let diagnostics = profile!(
        "canon.corsa.map_diagnostics",
        map_batch_diagnostics(raw_diagnostics, project)
    );
    let success = diagnostics
        .iter()
        .all(|diagnostic| diagnostic.severity != 1);
    Ok(TypeCheckResult {
        exit_code: if success { 0 } else { 1 },
        success,
        diagnostics,
    })
}

pub(super) fn collect_virtual_file_uris(virtual_root: &Path) -> CorsaResult<Vec<String>> {
    let mut uris = Vec::new();
    for entry in walkdir::WalkDir::new(virtual_root) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && !is_under_virtual_node_modules(virtual_root, path)
            && !is_internal_virtual_project_stub(path)
            && is_diagnostic_input(path)
        {
            uris.push(path_to_file_uri(path));
        }
    }
    uris.sort();
    Ok(uris)
}

fn is_diagnostic_input(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "mts" | "cts")
    )
}

fn is_internal_virtual_project_stub(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                AUTO_IMPORT_STUBS_FILE | VUE_MODULE_STUBS_FILE | SHARED_HELPERS_FILE
            )
        })
}

fn is_under_virtual_node_modules(virtual_root: &Path, path: &Path) -> bool {
    path.strip_prefix(virtual_root)
        .ok()
        .and_then(|path| path.components().next())
        .and_then(|component| component.as_os_str().to_str())
        .is_some_and(|name| name == "node_modules")
}

#[cfg(test)]
mod tests;
