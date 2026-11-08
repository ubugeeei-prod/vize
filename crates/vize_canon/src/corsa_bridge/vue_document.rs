//! Vue virtual-document synchronization for editor Corsa sessions.
#![expect(
    clippy::disallowed_types,
    reason = "warm hover shares one immutable projection across repeated requests"
)]

use std::path::{Path, PathBuf};

use vize_carton::String;

use super::bridge::CorsaBridge;
use super::types::CorsaBridgeError;
use super::vue_dependencies::{collect_dependency_documents, tsx_vue_import_shim};
use crate::batch::{ImportRewriter, VueDocumentVirtualTsOptions};
use crate::file_uri::path_to_file_uri;
use crate::virtual_ts::VirtualTsOptions;

pub(in crate::corsa_bridge) mod build;
pub(super) mod materialized_documents;
use build::{
    build_vue_virtual_workspace_project, build_vue_virtual_workspace_project_with_overlay_snapshot,
};
mod hover_document;
mod initial_project;
mod model;
pub(in crate::corsa_bridge) mod overlay_snapshot;
pub use hover_document::CorsaVueHoverDocument;
pub use initial_project::{CorsaInitialVueDocument, CorsaInitialVueProject};
mod prepare;
pub(super) use model::GeneratedVueDocument;
pub use model::{
    CorsaMaterializedMappingKind, CorsaMaterializedSource, CorsaVueVirtualDependency,
    CorsaVueVirtualDocument, CorsaVueVirtualDocumentOptions,
};
pub(crate) use model::{CorsaNativeDocumentBatch, CorsaVueVirtualProject};
pub use overlay_snapshot::CorsaOverlaySnapshot;
pub(crate) use overlay_snapshot::OverlaySources;
pub(in crate::corsa_bridge) use overlay_snapshot::{OverlayFingerprint, PersistentOverlayMap};
pub use prepare::CorsaVueWorkspaceRequest;

pub(crate) use model::CorsaProjectEnvironment;

impl CorsaBridge {
    /// Forget retired source membership without waiting for backend IPC.
    /// The next project synchronization computes its real removed-source delta.
    pub fn forget_cached_vue_virtual_documents(&self, source_paths: &[PathBuf]) {
        super::vue_dependencies_alias::AliasContext::forget_cached_sources(
            &self.editor_session,
            source_paths,
        );
    }

    /// Remove virtual TypeScript overlays derived from deleted Vue SFCs.
    pub async fn forget_vue_virtual_documents(
        &self,
        source_paths: &[PathBuf],
    ) -> Result<(), CorsaBridgeError> {
        self.forget_cached_vue_virtual_documents(source_paths);
        let source_paths = source_paths.to_vec();
        self.with_client(move |client| {
            client
                .forget_vue_virtual_documents(&source_paths)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }

    /// Generate, sync, and return the canonical `.vue.{ts,tsx}` document used
    /// for editor diagnostics, hover, definition, references, and rename.
    pub async fn open_vue_virtual_document(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        self.open_vue_virtual_document_with_overlays(source_path, content, options, &[])
            .await
    }

    /// Generate and sync a Vue document while preferring unsaved dependency
    /// buffers over their on-disk contents.
    pub async fn open_vue_virtual_document_with_overlays(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, String)],
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        self.open_vue_virtual_document_with_overlays_and_options(
            source_path,
            content,
            options,
            overlays,
            &VirtualTsOptions::default(),
        )
        .await
    }

    /// Generate and sync a Vue document with editor-specific virtual-TS options.
    pub async fn open_vue_virtual_document_with_overlays_and_options(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, String)],
        virtual_ts_options: &VirtualTsOptions,
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        let overlays = overlays
            .iter()
            .map(|(path, content)| (path.clone(), content.as_str()))
            .collect::<Vec<_>>();
        self.open_vue_virtual_document_with_borrowed_overlays_and_options(
            source_path,
            content,
            options,
            &overlays,
            virtual_ts_options,
        )
        .await
    }

    /// Generate and sync a Vue document without copying unchanged overlay text.
    /// Reachable and registered sources share a revision with borrowed text.
    pub async fn open_vue_virtual_document_with_borrowed_overlays_and_options(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, &str)],
        virtual_ts_options: &VirtualTsOptions,
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        self.open_vue_virtual_workspace_document(
            source_path,
            content,
            options,
            overlays,
            virtual_ts_options,
            &[],
        )
        .await
    }

    /// Register a complete query surface and synchronize one native revision.
    /// Workspace references should not rebuild the project once per SFC.
    pub async fn open_vue_virtual_workspace_document(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, &str)],
        virtual_ts_options: &VirtualTsOptions,
        requested_sources: &[(PathBuf, &str)],
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        let (host, source_catalog) = self
            .open_vue_virtual_workspace_project(
                source_path,
                content,
                options,
                overlays,
                None,
                virtual_ts_options,
                requested_sources,
            )
            .await?;
        let mut host = (*host).clone();
        host.source_catalog = source_catalog;
        Ok(host)
    }

    /// Synchronize a Vue revision while reusing its precomputed overlay
    /// identities across every semantic request.
    pub async fn open_vue_virtual_workspace_document_with_overlay_snapshot(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlay_snapshot: &CorsaOverlaySnapshot,
        virtual_ts_options: &VirtualTsOptions,
        requested_sources: &[(PathBuf, &str)],
    ) -> Result<CorsaVueVirtualDocument, CorsaBridgeError> {
        let (host, source_catalog) = self
            .open_vue_virtual_workspace_project(
                source_path,
                content,
                options,
                &[],
                Some(overlay_snapshot),
                virtual_ts_options,
                requested_sources,
            )
            .await?;
        let mut host = (*host).clone();
        host.source_catalog = source_catalog;
        Ok(host)
    }

    /// Synchronize one local hover projection without copying its immutable
    /// generated text, mappings, dependencies, or materialized sources.
    pub async fn open_vue_hover_document_with_borrowed_overlays_and_options(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, &str)],
        virtual_ts_options: &VirtualTsOptions,
    ) -> Result<CorsaVueHoverDocument, CorsaBridgeError> {
        let (host, _) = self
            .open_vue_virtual_workspace_project(
                source_path,
                content,
                options,
                overlays,
                None,
                virtual_ts_options,
                &[],
            )
            .await?;
        Ok(CorsaVueHoverDocument::new(host))
    }

    /// Borrow the immutable hover surface and its revision-scoped overlay
    /// identities without rehashing every open buffer.
    pub async fn open_vue_hover_document_with_overlay_snapshot(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlay_snapshot: &CorsaOverlaySnapshot,
        virtual_ts_options: &VirtualTsOptions,
    ) -> Result<CorsaVueHoverDocument, CorsaBridgeError> {
        let (host, _) = self
            .open_vue_virtual_workspace_project(
                source_path,
                content,
                options,
                &[],
                Some(overlay_snapshot),
                virtual_ts_options,
                &[],
            )
            .await?;
        Ok(CorsaVueHoverDocument::new(host))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one private boundary binds source, overlay revision, generation and workspace inputs"
    )]
    async fn open_vue_virtual_workspace_project(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, &str)],
        overlay_snapshot: Option<&CorsaOverlaySnapshot>,
        virtual_ts_options: &VirtualTsOptions,
        requested_sources: &[(PathBuf, &str)],
    ) -> Result<
        (
            std::sync::Arc<CorsaVueVirtualDocument>,
            super::CorsaSourceCatalog,
        ),
        CorsaBridgeError,
    > {
        let project = vize_l0::profile!(
            "canon.editor.project_prepare",
            match overlay_snapshot {
                Some(snapshot) => build_vue_virtual_workspace_project_with_overlay_snapshot(
                    source_path,
                    content,
                    options,
                    snapshot,
                    requested_sources,
                    CorsaProjectEnvironment {
                        virtual_ts_options,
                        package_routes: &self.package_route_resolver,
                        project_root: self.config.working_dir.as_deref(),
                        tsconfig_path: self.config.tsconfig_path.as_deref(),
                        editor_session: &self.editor_session,
                    },
                ),
                None => build_vue_virtual_workspace_project(
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
                ),
            }
        )?;
        let CorsaVueVirtualProject {
            host,
            source_catalog,
            mut documents,
            session_project_root,
            session_config_path,
            materialized_changes,
        } = project;
        if let Some(project_root) = session_project_root.as_deref() {
            source_catalog.extend_native_project_document_batch(project_root, &mut documents);
        }
        self.open_canon_project_documents(
            documents,
            session_project_root,
            session_config_path,
            materialized_changes,
        )
        .await?;
        Ok((host, source_catalog))
    }

    pub(super) async fn open_canon_project_documents(
        &self,
        documents: impl Into<CorsaNativeDocumentBatch>,
        session_project_root: Option<PathBuf>,
        session_config_path: Option<PathBuf>,
        materialized_changes: crate::batch::virtual_project::MaterializedFileDelta,
    ) -> Result<(), CorsaBridgeError> {
        let documents = documents.into();
        let phase = super::preparation_trace::Phase::start("project_synchronize", documents.len());
        let timer = self.profiler().timer("corsa_project_synchronize");
        let cache_len = self
            .with_client(move |client| {
                if let Some(project_root) = session_project_root {
                    vize_l0::profile!("canon.editor.project_activate", {
                        client
                            .synchronize_materialized_project(
                                &project_root,
                                session_config_path.as_deref(),
                                &materialized_changes,
                            )
                            .map_err(CorsaBridgeError::CommunicationError)
                    })?;
                }
                vize_l0::profile!("canon.editor.native_batch_open", {
                    client
                        .did_open_immutable_batch(&documents)
                        .map_err(CorsaBridgeError::CommunicationError)
                })?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats().set_entries(cache_len as u64);
        phase.finish();
        if let Some(timer) = timer {
            timer.record(self.profiler());
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn build_vue_virtual_project(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
    build_vue_virtual_project_with_overlays(source_path, content, options, &[])
}

#[cfg(test)]
pub(crate) fn build_vue_virtual_project_with_overlays(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &[(PathBuf, &str)],
) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
    build_vue_virtual_project_with_overlays_and_options(
        source_path,
        content,
        options,
        overlays,
        &VirtualTsOptions::default(),
    )
}

#[cfg(test)]
fn build_vue_virtual_project_with_overlays_and_options(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &[(PathBuf, &str)],
    virtual_ts_options: &VirtualTsOptions,
) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
    build_vue_virtual_project_with_overlays_and_options_and_package_routes(
        source_path,
        content,
        options,
        overlays,
        CorsaProjectEnvironment {
            virtual_ts_options,
            package_routes: &crate::PackageRouteResolver::default(),
            project_root: None,
            tsconfig_path: None,
            editor_session: super::editor_session::fallback_editor_session(),
        },
    )
}

pub(crate) fn build_vue_virtual_project_with_overlays_and_options_and_package_routes(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &[(PathBuf, &str)],
    environment: CorsaProjectEnvironment<'_>,
) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
    build_vue_virtual_workspace_project(source_path, content, options, overlays, &[], environment)
}

/// Generate Vue output with offset-preserving Canon alias identities (#3900).
pub(super) fn generate_vue_document_with_alias(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    context: &super::vue_dependencies_alias::AliasContext,
) -> Result<GeneratedVueDocument, CorsaBridgeError> {
    generate_vue_document_with_options(
        source_path,
        content,
        options,
        context.virtual_ts_options(),
        rewriter,
        Some(context),
    )
}

mod generate;
use generate::generate_vue_document_with_options;
