//! Read one mapped diagnostic product on the same native worker as its project update.
use super::{
    CorsaBridge, CorsaBridgeError, CorsaProjectEnvironment, CorsaVueVirtualDocumentOptions,
    CorsaVueVirtualProject, VirtualTsOptions, build_vue_virtual_workspace_project,
};
use crate::corsa_bridge::{
    PhasedLspDiagnostic, bridge::convert_bridge_diagnostics_with_provenance,
};
use std::path::{Path, PathBuf};

/// The fields consumed by the common diagnostic assembler for one original host.
pub struct AmbientVueDiagnostics {
    pub code: vize_l0::String,
    pub mapping: crate::virtual_ts::ProjectionMapping,
    pub import_source_map: crate::batch::ImportSourceMap,
    pub diagnostics: Vec<PhasedLspDiagnostic>,
    pub snapshot: Option<crate::lsp_client::AmbientDiagnosticSnapshot>,
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
        let CorsaVueVirtualProject {
            host,
            source_catalog,
            mut documents,
            session_project_root,
            session_config_path,
            materialized_changes,
        } = build_vue_virtual_workspace_project(
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
        if let Some(root) = session_project_root.as_deref() {
            source_catalog.extend_native_project_document_batch(root, &mut documents);
        }
        let uri = host.request_uri.clone();
        let (used_cache, cache_len, diagnostics, snapshot) = self
            .with_client(move |client| {
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
                let (fetch, snapshot) = client
                    .read_ambient_diagnostic_snapshot(&uri, previous)
                    .map_err(CorsaBridgeError::CommunicationError)?;
                let diagnostics = convert_bridge_diagnostics_with_provenance(&fetch.diagnostics)?;
                Ok((
                    fetch.used_cache,
                    client.diagnostics_cache_len(),
                    diagnostics,
                    snapshot,
                ))
            })
            .await?;
        self.cache_stats().set_entries(cache_len as u64);
        if used_cache {
            self.cache_stats().hit();
        } else {
            self.cache_stats().miss();
        }
        Ok(AmbientVueDiagnostics {
            code: host.code.clone(),
            mapping: host.mapping.clone(),
            import_source_map: host.import_source_map.clone(),
            diagnostics,
            snapshot,
        })
    }
}
