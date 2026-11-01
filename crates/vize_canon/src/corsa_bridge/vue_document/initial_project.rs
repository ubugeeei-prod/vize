//! Owned cold-only Canon graph; it never represents an initialized process.
#![expect(clippy::disallowed_types, reason = "owned seed crosses the worker")]

use std::sync::{Arc, atomic::Ordering};

use super::{
    CorsaBridge, CorsaBridgeError, CorsaProjectEnvironment, CorsaVueVirtualDocument,
    CorsaVueVirtualProject, CorsaVueWorkspaceRequest, build_vue_virtual_workspace_project,
};
use crate::corsa_bridge::bridge::startup::{StartupLease, StartupReceipt};
use crate::corsa_bridge::session::build_initial_project_client;

/// Opaque admission bound to one bridge, editor mirror and stop epoch. Dropping
/// an unstarted seed rolls back only its exclusively claimed editor revision.
pub struct CorsaInitialVueProject {
    project: CorsaVueVirtualProject,
    lease: StartupLease,
    config_bytes: Vec<u8>,
}

/// One request-owned native input receipt. It is consumed only after the
/// original graph builder has validated the complete current revision again.
pub struct CorsaInitialVueDocument {
    project: CorsaVueVirtualProject,
    owner: StartupReceipt,
    operation_epoch: u64,
    config_bytes: Vec<u8>,
}

impl CorsaBridge {
    /// Prepare the original complete workspace request before any native
    /// process starts. `None` keeps unadmitted/occupied callers on eager startup.
    pub fn prepare_initial_vue_project(
        &self,
        request: CorsaVueWorkspaceRequest,
    ) -> Result<Option<CorsaInitialVueProject>, CorsaBridgeError> {
        let Some(authored) = self.config.working_dir.as_deref() else {
            return Ok(None);
        };
        if !self.shared_editor_transport
            || !(authored.join("tsconfig.json").is_file()
                || authored.join("jsconfig.json").is_file())
        {
            return Ok(None);
        }
        let Some(lease) = self.startup.reserve(
            Arc::clone(&self.editor_session),
            true,
            self.is_initialized(),
        ) else {
            return Ok(None);
        };
        let mut project = self.build_owned_initial_request(&request)?;
        let (Some(root), Some(_)) = (
            project.session_project_root.as_deref(),
            project.session_config_path.as_deref(),
        ) else {
            return Ok(None);
        };
        project
            .host
            .source_catalog
            .extend_native_project_documents(root, &mut project.documents);
        // This guard is pure UTF-8/URI validation. The factory repeats it
        // defensively before real startup; it performs no RPC here.
        crate::corsa_client::CorsaProjectClient::validate_initial_editor_project_delta(
            &project.materialized_changes,
        )
        .map_err(CorsaBridgeError::CommunicationError)?;
        if !lease.is_current() {
            return Ok(None);
        }
        let config_bytes = read_config(&project)?;
        Ok(Some(CorsaInitialVueProject {
            project,
            lease,
            config_bytes,
        }))
    }

    /// Perform a real direct-Canon handshake and complete owned document open
    /// on the original FIFO worker/deadline. `current` is the caller's original
    /// environment admission predicate, not a fresh document/request stamp.
    pub async fn spawn_with_initial_project<F>(
        &self,
        seed: CorsaInitialVueProject,
        current: F,
    ) -> Result<CorsaInitialVueDocument, CorsaBridgeError>
    where
        F: Fn() -> bool + Send + 'static,
    {
        if !seed.lease.belongs_to(&self.startup, &self.editor_session) {
            return Err(CorsaBridgeError::NotInitialized);
        }
        let config = self.config.clone();
        let initialized = Arc::clone(&self.initialized);
        let operation_epoch = Arc::clone(&self.client_operation_epoch);
        let (receipt, cache_len) = self
            .submit_async(move |slot| {
                let CorsaInitialVueProject {
                    project,
                    mut lease,
                    config_bytes,
                } = seed;
                if !current() || slot.is_some() || !lease.is_current() {
                    return Err(CorsaBridgeError::ProcessTerminated);
                }
                let CorsaVueVirtualProject {
                    host,
                    documents,
                    session_project_root,
                    session_config_path,
                    materialized_changes,
                } = project;
                let root = session_project_root.ok_or(CorsaBridgeError::NotInitialized)?;
                let config_path = session_config_path.ok_or(CorsaBridgeError::NotInitialized)?;
                if !lease.enter() {
                    return Err(CorsaBridgeError::ProcessTerminated);
                }
                let outcome = (|| {
                    *slot = Some(build_initial_project_client(
                        &config,
                        &root,
                        &config_path,
                        &materialized_changes,
                    )?);
                    lease.retain();
                    if !current() || !lease.is_current() {
                        return Err(CorsaBridgeError::ProcessTerminated);
                    }
                    let client = slot.as_mut().ok_or(CorsaBridgeError::NotInitialized)?;
                    let borrowed = documents
                        .iter()
                        .map(|(uri, text)| (uri.as_str(), text.as_str()))
                        .collect::<Vec<_>>();
                    client
                        .did_open_batch_fast(&borrowed)
                        .map_err(CorsaBridgeError::CommunicationError)?;
                    let cache_len = client.diagnostics_cache_len();
                    lease.publish(&initialized, current())?;
                    Ok((
                        CorsaInitialVueDocument {
                            project: CorsaVueVirtualProject {
                                host,
                                documents,
                                session_project_root: Some(root),
                                session_config_path: Some(config_path),
                                materialized_changes,
                            },
                            owner: lease.receipt(),
                            operation_epoch: operation_epoch.load(Ordering::SeqCst),
                            config_bytes,
                        },
                        cache_len,
                    ))
                })();
                if outcome.is_err() {
                    // A failed constructor can still have an unknown-live
                    // native transport. Neither last-Arc Drop nor a later
                    // apparently successful cleanup proves its old readers gone.
                    lease.retain_after_failure();
                }
                outcome
            })
            .await?;
        self.cache_stats().set_entries(cache_len as u64);
        Ok(receipt)
    }

    /// Validate the original complete graph again before reusing the initial
    /// native open. Any input, owner, catalog or delta mismatch takes the
    /// ordinary synchronization path and returns that newly validated host.
    #[expect(
        clippy::too_many_arguments,
        reason = "existing borrowed workspace arguments plus one owned initial receipt"
    )]
    pub async fn open_initial_vue_project(
        &self,
        receipt: CorsaInitialVueDocument,
        source_path: &std::path::Path,
        content: &str,
        options: super::CorsaVueVirtualDocumentOptions,
        overlays: &[(std::path::PathBuf, &str)],
        virtual_ts_options: &crate::virtual_ts::VirtualTsOptions,
        requested_sources: &[(std::path::PathBuf, &str)],
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        let mut project = build_vue_virtual_workspace_project(
            source_path,
            content,
            options,
            overlays,
            requested_sources,
            CorsaProjectEnvironment {
                virtual_ts_options,
                package_routes: &self.package_route_resolver,
                project_root: self.config.working_dir.as_deref(),
                tsconfig_path: self.config.tsconfig_path.as_deref(),
                editor_session: &self.editor_session,
            },
        )?;
        if let Some(root) = project.session_project_root.as_deref() {
            project
                .host
                .source_catalog
                .extend_native_project_documents(root, &mut project.documents);
        }
        let inputs_identical = receipt.owner.is_current(self)
            && receipt.matches_config_read(&project, read_config(&project));
        let identical = if inputs_identical {
            let expected = receipt.operation_epoch;
            let epoch = Arc::clone(&self.client_operation_epoch);
            let initialized = Arc::clone(&self.initialized);
            let owner = receipt.owner;
            self.submit_ready_async(move |slot| {
                // The wrapper increments once for this conservative probe.
                // Compare its exact predecessor on the SAME native FIFO lane;
                // even a metadata/read/error operation invalidates reuse.
                Ok(slot.is_some()
                    && owner.is_ready(&initialized)
                    && epoch.load(Ordering::SeqCst) == expected.wrapping_add(1))
            })
            .await?
        } else {
            false
        };
        let CorsaVueVirtualProject {
            host,
            documents,
            session_project_root,
            session_config_path,
            materialized_changes,
        } = project;
        if !identical {
            self.open_canon_project_documents(
                documents,
                session_project_root,
                session_config_path,
                materialized_changes,
            )
            .await?;
        }
        Ok(host)
    }

    fn build_owned_initial_request(
        &self,
        request: &CorsaVueWorkspaceRequest,
    ) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
        let overlays = request
            .overlays
            .iter()
            .map(|(path, text)| (path.clone(), &**text))
            .collect::<Vec<_>>();
        let requested = request
            .requested_sources
            .iter()
            .map(|(path, text)| (path.clone(), &**text))
            .collect::<Vec<_>>();
        build_vue_virtual_workspace_project(
            &request.source_path,
            &request.content,
            request.options,
            &overlays,
            &requested,
            CorsaProjectEnvironment {
                virtual_ts_options: &request.virtual_ts_options,
                package_routes: &self.package_route_resolver,
                project_root: self.config.working_dir.as_deref(),
                tsconfig_path: self.config.tsconfig_path.as_deref(),
                editor_session: &self.editor_session,
            },
        )
    }
}

impl CorsaInitialVueDocument {
    fn matches_config_read(
        &self,
        project: &CorsaVueVirtualProject,
        bytes: Result<Vec<u8>, CorsaBridgeError>,
    ) -> bool {
        // Failure to prove reuse never replaces the ordinary synchronization
        // operation or its original error/fallback behavior.
        bytes.is_ok_and(|bytes| self.matches(project, &bytes))
    }

    fn matches(&self, project: &CorsaVueVirtualProject, config_bytes: &[u8]) -> bool {
        self.project.session_project_root == project.session_project_root
            && self.project.session_config_path == project.session_config_path
            && self.config_bytes == config_bytes
            && self.project.documents == project.documents
            && self
                .project
                .host
                .source_catalog
                .shares_revision_with(&project.host.source_catalog)
            && ((!project.materialized_changes.has_topology_changes()
                && project.materialized_changes.is_empty())
                || self.project.materialized_changes == project.materialized_changes)
    }
}

fn read_config(project: &CorsaVueVirtualProject) -> Result<Vec<u8>, CorsaBridgeError> {
    match &project.session_config_path {
        Some(path) => std::fs::read(path).map_err(|error| {
            CorsaBridgeError::CommunicationError(vize_carton::cstr!(
                "Failed to read initial Canon config: {error}"
            ))
        }),
        None => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, target_os = "linux"))]
mod native_tests;
