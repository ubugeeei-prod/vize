//! Read one mapped diagnostic product on the same native worker as its project update.
use super::{
    CorsaBridge, CorsaBridgeError, CorsaProjectEnvironment, CorsaVueVirtualDocumentOptions,
    CorsaVueVirtualProject, VirtualTsOptions, build_vue_virtual_workspace_project,
};
use crate::corsa_bridge::{
    PhasedLspDiagnostic, bridge::convert_bridge_diagnostics_with_provenance,
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::Duration,
};

/// The fields consumed by the common diagnostic assembler for one original host.
pub struct AmbientVueDiagnostics {
    pub code: vize_l0::String,
    pub mapping: crate::virtual_ts::ProjectionMapping,
    pub import_source_map: crate::batch::ImportSourceMap,
    pub diagnostics: Vec<PhasedLspDiagnostic>,
    pub snapshot: Option<crate::lsp_client::AmbientDiagnosticSnapshot>,
}

/// The source and workspace inputs to one native document read.
pub struct AmbientVueDiagnosticRequest<'a> {
    pub source_path: &'a Path,
    pub content: &'a str,
    pub options: CorsaVueVirtualDocumentOptions,
    pub overlays: &'a [(PathBuf, &'a str)],
    pub virtual_ts_options: &'a VirtualTsOptions,
    pub requested_sources: &'a [(PathBuf, &'a str)],
}

struct NativeBatch {
    host: Arc<super::CorsaVueVirtualDocument>,
    diagnostics: Vec<PhasedLspDiagnostic>,
    reused_projection: bool,
    snapshot: Option<crate::lsp_client::AmbientDiagnosticSnapshot>,
}

/// A validated native batch awaiting its caller's original-source projection.
pub struct AmbientVueDiagnosticBatch<T> {
    batch: NativeBatch,
    previous: Option<T>,
}

/// The mapped value and the exact native and generated documents that produced it.
pub struct AmbientVueDiagnosticRead<T> {
    value: T,
    host: std::sync::Weak<super::CorsaVueVirtualDocument>,
    snapshot: Option<crate::lsp_client::AmbientDiagnosticSnapshot>,
}

type Owner = (Option<crate::lsp_client::AmbientDiagnosticSnapshot>, bool);

enum SynchronizedOwner {
    Ready(Owner),
    Pending(mpsc::Receiver<Owner>, Duration),
}

impl SynchronizedOwner {
    fn finish(self) -> Owner {
        match self {
            Self::Ready(owner) => owner,
            Self::Pending(response, timeout) => {
                response.recv_timeout(timeout).unwrap_or((None, false))
            }
        }
    }
}

impl<T> AmbientVueDiagnosticRead<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn has_native_snapshot(&self) -> bool {
        self.snapshot.is_some()
    }

    /// Resume the generated-row API. It will read every row for this cursor.
    pub fn into_native_snapshot(self) -> Option<crate::lsp_client::AmbientDiagnosticSnapshot> {
        self.snapshot
    }
}

impl<T> AmbientVueDiagnosticBatch<T> {
    /// Project fresh rows once, or return the value already attached to this world.
    pub fn map(
        self,
        project: impl FnOnce(&super::CorsaVueVirtualDocument, Vec<PhasedLspDiagnostic>) -> T,
    ) -> AmbientVueDiagnosticRead<T> {
        let batch = self.batch;
        let value = match self.previous {
            Some(value) if batch.reused_projection => value,
            _ => project(&batch.host, batch.diagnostics),
        };
        AmbientVueDiagnosticRead {
            value,
            host: Arc::downgrade(&batch.host),
            snapshot: batch.snapshot.map(|snapshot| snapshot.into_mapped_cursor()),
        }
    }
}

impl CorsaBridge {
    /// Validate the full Canon revision and read fresh native diagnostics in the
    /// same transaction that admits its documents. Only host mapping data crosses
    /// the consumer boundary.
    #[expect(
        clippy::too_many_arguments,
        reason = "borrowed workspace inputs and one prior native read"
    )]
    pub async fn read_ambient_vue_diagnostics(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, &str)],
        virtual_ts_options: &VirtualTsOptions,
        requested_sources: &[(PathBuf, &str)],
        previous: Option<crate::lsp_client::AmbientDiagnosticSnapshot>,
    ) -> Result<AmbientVueDiagnostics, CorsaBridgeError> {
        let batch = self
            .read_native_ambient_batch(
                AmbientVueDiagnosticRequest {
                    source_path,
                    content,
                    options,
                    overlays,
                    virtual_ts_options,
                    requested_sources,
                },
                previous,
                None,
            )
            .await?;
        Ok(AmbientVueDiagnostics {
            code: batch.host.code.clone(),
            mapping: batch.host.mapping.clone(),
            import_source_map: batch.host.import_source_map.clone(),
            diagnostics: batch.diagnostics,
            snapshot: batch.snapshot,
        })
    }

    /// Bind the caller's mapped answer to the host and native read that produced it.
    pub async fn read_ambient_vue_diagnostic_batch<T>(
        &self,
        request: AmbientVueDiagnosticRequest<'_>,
        previous: Option<AmbientVueDiagnosticRead<T>>,
    ) -> Result<AmbientVueDiagnosticBatch<T>, CorsaBridgeError> {
        let (native, projection) = previous
            .and_then(|read| Some((read.snapshot?, (read.host, read.value))))
            .unzip();
        let (host, previous) = projection.unzip();
        let batch = self
            .read_native_ambient_batch(request, native, host)
            .await?;
        Ok(AmbientVueDiagnosticBatch { batch, previous })
    }

    async fn read_native_ambient_batch(
        &self,
        request: AmbientVueDiagnosticRequest<'_>,
        previous: Option<crate::lsp_client::AmbientDiagnosticSnapshot>,
        previous_host: Option<std::sync::Weak<super::CorsaVueVirtualDocument>>,
    ) -> Result<NativeBatch, CorsaBridgeError> {
        let AmbientVueDiagnosticRequest {
            source_path,
            content,
            options,
            overlays,
            virtual_ts_options,
            requested_sources,
        } = request;
        // Ask the immutable native snapshot about its own project while preparing
        // the full current workspace. The mutable client checks the lease, document
        // and project again after every materialized change has been synchronized.
        let (owner, project) = futures::join!(self.confirm_ambient_owner(previous), async {
            build_vue_virtual_workspace_project(
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
            )
        });
        let (previous, confirmed_owner) = owner;
        let CorsaVueVirtualProject {
            host,
            source_catalog,
            mut documents,
            session_project_root,
            session_config_path,
            materialized_changes,
        } = project?;
        if let Some(root) = session_project_root.as_deref() {
            source_catalog.extend_native_project_document_batch(root, &mut documents);
        }
        let same_projection = previous_host
            .and_then(|old| old.upgrade())
            .is_some_and(|old| Arc::ptr_eq(&old, &host));
        let native_host = Arc::clone(&host);
        let synchronize = |owner: SynchronizedOwner| {
            self.with_client(move |client| {
                if let Some(root) = session_project_root {
                    client
                        .synchronize_materialized_project(
                            &root,
                            session_config_path.as_deref(),
                            &materialized_changes,
                        )
                        .map_err(CorsaBridgeError::CommunicationError)?;
                }
                client
                    .did_open_immutable_batch(&documents)
                    .map_err(CorsaBridgeError::CommunicationError)?;
                let (mut snapshot, confirmed_owner) = owner.finish();
                crate::corsa_bridge::native_operation::checkpoint()
                    .map_err(CorsaBridgeError::CommunicationError)?;
                let (diagnostics, origin) = {
                    let (rows, origin) = client
                        .read_ambient_diagnostic_snapshot(
                            &native_host.request_uri,
                            &mut snapshot,
                            confirmed_owner,
                            same_projection,
                        )
                        .map_err(CorsaBridgeError::CommunicationError)?;
                    (convert_bridge_diagnostics_with_provenance(&rows)?, origin)
                };
                Ok((
                    origin,
                    client.diagnostics_cache_len(),
                    diagnostics,
                    snapshot,
                ))
            })
        };
        let result = match previous {
            Some(previous) if previous.has_empty_native_rows() && !confirmed_owner => {
                // Workspace construction has released its transients. Overlap the immutable
                // owner RPC with the native file synchronization, which still precedes the
                // world and host checks in this single mutable transaction.
                let (reply, response) = mpsc::sync_channel(1);
                let timeout = Duration::from_millis(self.config.timeout_ms.max(1));
                let probe = async move {
                    let answer = self.probe_ambient_owner(previous).await;
                    let _ = reply.send(answer);
                    Ok(())
                };
                futures::try_join!(
                    probe,
                    synchronize(SynchronizedOwner::Pending(response, timeout))
                )
                .map(|(_, result)| result)
            }
            previous => synchronize(SynchronizedOwner::Ready((previous, confirmed_owner))).await,
        };
        let (origin, cache_len, diagnostics, snapshot) = result?;
        self.cache_stats().set_entries(cache_len as u64);
        if origin.used_cache() {
            self.cache_stats().hit();
        } else {
            self.cache_stats().miss();
        }
        Ok(NativeBatch {
            host,
            reused_projection: same_projection && origin.confirmed_previous(),
            diagnostics,
            snapshot,
        })
    }
}
