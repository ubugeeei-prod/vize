//! Corsa project-session execution, including persistent incremental snapshots.

use std::path::Path;

use crate::{corsa_client::CorsaProjectClient, file_uri::path_to_file_uri};
use vize_carton::{String, profile};

use super::{
    CorsaExecutor, FallbackStep, MaterializeLock, TypeCheckResult, VirtualProject, check_with_cli,
    map_corsa_error, should_fallback_to_cli, warn_fallback,
};
use crate::batch::error::CorsaResult;
use crate::batch::executor::diagnostics::map_batch_diagnostics;
use crate::batch::source_policy::SourceFilePolicy;
use crate::batch::type_checker::IncrementalCheckMetrics;
use crate::batch::virtual_project::IncrementalMaterialization;
use crate::batch::virtual_project::{
    AUTO_IMPORT_STUBS_FILE, SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE,
};

mod diagnostic_paths;
mod snapshot;

use diagnostic_paths::extend_diagnostic_path_uris;
#[cfg(test)]
use snapshot::MaterializedDelta;
use snapshot::MaterializedSnapshot;

#[derive(Default)]
pub(super) struct IncrementalSessionState {
    session: Option<IncrementalSession>,
    pub(super) metrics: IncrementalCheckMetrics,
}

struct IncrementalSession {
    client: CorsaProjectClient,
    snapshot: MaterializedSnapshot,
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
            collect_virtual_file_uris(project.virtual_root(), project.source_file_policy())
        )?;
        extend_diagnostic_path_uris(project, &mut uris);
        check_session_client(&mut client, project, &uris)
    }

    pub(crate) fn check_incremental_session(
        &self,
        project: &mut VirtualProject,
        servers: Option<usize>,
    ) -> CorsaResult<TypeCheckResult> {
        let session_result = {
            let _materialize_lock = MaterializeLock::acquire(project.virtual_root())?;
            let has_session = self
                .incremental_session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .session
                .is_some();
            let prepared = if has_session {
                Some(profile!(
                    "canon.executor.materialize_incremental_delta",
                    project.materialize_incremental_delta()
                )?)
            } else {
                profile!(
                    "canon.executor.materialize_incremental",
                    project.materialize()
                )?;
                project.capture_materialized_package_links();
                project.discard_incremental_materialization();
                None
            };
            let mut state = self
                .incremental_session
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let result = if let Some(prepared) = prepared {
                state.check_delta(project, prepared)
            } else {
                let mut snapshot = profile!(
                    "canon.corsa.incremental.snapshot",
                    MaterializedSnapshot::capture(
                        project.virtual_root(),
                        project.source_file_policy()
                    )
                )?;
                snapshot.extend_diagnostic_paths(project)?;
                state.check(&self.corsa_path, project, snapshot)
            };
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
        self.metrics.last_materialized_entries_considered = snapshot.revisions.len();
        self.metrics.last_tree_entries_scanned = snapshot.revisions.len();
        self.metrics.last_full_rebuild = self.session.is_none();
        self.metrics.last_source_nodes_rebuilt = 0;
        self.metrics.last_dependency_nodes_reconciled = 0;
        self.metrics.last_shadow_bindings_rebuilt = 0;
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

    fn check_delta(
        &mut self,
        project: &VirtualProject,
        prepared: IncrementalMaterialization,
    ) -> CorsaResult<TypeCheckResult> {
        let session = self
            .session
            .as_mut()
            .ok_or(crate::batch::error::CorsaError::NotInitialized)?;
        self.metrics.checks += 1;
        self.metrics.last_session_started = false;
        self.metrics.last_session_reused = true;
        self.metrics.last_session_to_cli_fallback = false;
        self.metrics.last_changed_files = prepared.delta.changed.len();
        self.metrics.last_created_files = prepared.delta.created.len();
        self.metrics.last_deleted_files = prepared.delta.deleted.len();
        self.metrics.last_materialized_entries_considered = prepared.considered;
        self.metrics.last_tree_entries_scanned = 0;
        self.metrics.last_full_rebuild = false;
        self.metrics.last_full_rebuild = prepared.full_topology_rebuild;
        self.metrics.last_source_nodes_rebuilt = prepared.source_nodes_rebuilt;
        self.metrics.last_dependency_nodes_reconciled = prepared.dependency_nodes_reconciled;
        self.metrics.last_shadow_bindings_rebuilt = prepared.shadow_bindings_rebuilt;
        let refreshed = !prepared.delta.is_empty();
        self.metrics.last_session_refreshed = refreshed;
        if refreshed {
            profile!(
                "canon.corsa.incremental.refresh",
                session.client.refresh_materialized_files(
                    &prepared.delta.changed,
                    &prepared.delta.created,
                    &prepared.delta.deleted
                )
            )
            .map_err(map_corsa_error)?;
        }
        session.snapshot.apply_delta(project, &prepared.delta)?;
        self.metrics.last_requested_files = session.snapshot.uris.len();
        self.metrics.session_reuses += 1;
        self.metrics.session_refreshes += usize::from(refreshed);
        check_session_client(&mut session.client, project, &session.snapshot.uris)
    }
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

pub(super) fn collect_virtual_file_uris(
    virtual_root: &Path,
    source_policy: SourceFilePolicy,
) -> CorsaResult<Vec<String>> {
    let mut uris = Vec::new();
    for entry in walkdir::WalkDir::new(virtual_root) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && !is_under_virtual_node_modules(virtual_root, path)
            && !is_internal_virtual_project_stub(path)
            && source_policy.accepts_diagnostic_input(path)
        {
            uris.push(path_to_file_uri(path));
        }
    }
    uris.sort();
    Ok(uris)
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
