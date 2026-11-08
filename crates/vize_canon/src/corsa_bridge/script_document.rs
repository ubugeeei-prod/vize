//! Script-host synchronization for editor Corsa sessions.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, String};

use super::bridge::CorsaBridge;
use super::types::CorsaBridgeError;
use super::vue_dependencies::collect_script_dependency_documents;
use super::vue_document::CorsaVueVirtualDocumentOptions;
use crate::batch::ImportRewriter;

pub struct CorsaScriptVirtualDocumentRequest<'a> {
    pub source_path: &'a Path,
    pub request_path: &'a str,
    /// Input source; Canon owns JSX lowering when `options.jsx_typecheck` is enabled.
    pub code: &'a str,
    pub source_type: SourceType,
    pub options: CorsaVueVirtualDocumentOptions,
    pub overlays: &'a [(PathBuf, &'a str)],
    pub virtual_ts_options: &'a crate::virtual_ts::VirtualTsOptions,
}

struct BuiltScriptVirtualProject {
    host: CorsaScriptVirtualDocument,
    documents: Vec<(String, String)>,
    session_project_root: Option<PathBuf>,
    session_config_path: Option<PathBuf>,
    materialized_changes: crate::batch::virtual_project::MaterializedFileDelta,
    materialized_sources: Vec<super::CorsaMaterializedSource>,
    source_catalog: super::CorsaSourceCatalog,
}

/// The exact native script overlay and its import-coordinate transform.
pub struct CorsaScriptVirtualDocument {
    pub request_uri: String,
    pub code: String,
    /// Authored JSX/TSX coordinates when Canon lowered this document.
    pub mappings: Vec<crate::virtual_ts::VizeMapping>,
    pub import_source_map: crate::batch::ImportSourceMap,
    pub resolved_dependencies: Vec<PathBuf>,
}

/// Script projection and all source identities retained by the same project revision.
pub struct CorsaScriptVirtualProject {
    pub document: CorsaScriptVirtualDocument,
    pub materialized_sources: Vec<super::CorsaMaterializedSource>,
    pub source_catalog: super::CorsaSourceCatalog,
    pub session_project_root: Option<PathBuf>,
}

impl CorsaBridge {
    /// Open a generated TS/JS host together with every reachable Vue virtual
    /// document, using the same graph materialization as an SFC host.
    pub async fn open_script_virtual_document_with_vue_dependencies(
        &self,
        request: CorsaScriptVirtualDocumentRequest<'_>,
    ) -> Result<CorsaScriptVirtualDocument, CorsaBridgeError> {
        self.open_script_virtual_project_inner(request, None)
            .await
            .map(|project| project.document)
    }

    /// Open a script host against the persistent editor overlay revision.
    pub async fn open_script_virtual_document_with_overlay_snapshot(
        &self,
        request: CorsaScriptVirtualDocumentRequest<'_>,
        snapshot: &super::CorsaOverlaySnapshot,
    ) -> Result<CorsaScriptVirtualDocument, CorsaBridgeError> {
        self.open_script_virtual_project_inner(request, Some(snapshot))
            .await
            .map(|project| project.document)
    }

    /// Retain the exact project mappings for cross-file navigation from scripts.
    pub async fn open_script_virtual_project(
        &self,
        request: CorsaScriptVirtualDocumentRequest<'_>,
    ) -> Result<CorsaScriptVirtualProject, CorsaBridgeError> {
        self.open_script_virtual_project_inner(request, None).await
    }

    /// Retain script mappings without reconstructing the complete overlay map.
    pub async fn open_script_virtual_project_with_overlay_snapshot(
        &self,
        request: CorsaScriptVirtualDocumentRequest<'_>,
        snapshot: &super::CorsaOverlaySnapshot,
    ) -> Result<CorsaScriptVirtualProject, CorsaBridgeError> {
        self.open_script_virtual_project_inner(request, Some(snapshot))
            .await
    }

    async fn open_script_virtual_project_inner(
        &self,
        request: CorsaScriptVirtualDocumentRequest<'_>,
        snapshot: Option<&super::CorsaOverlaySnapshot>,
    ) -> Result<CorsaScriptVirtualProject, CorsaBridgeError> {
        let virtual_ts_options = request.virtual_ts_options;
        let project = build_script_virtual_project_with_package_routes(
            request,
            snapshot,
            super::vue_document::CorsaProjectEnvironment {
                virtual_ts_options,
                package_routes: &self.package_route_resolver,
                project_root: self.config.working_dir.as_deref(),
                tsconfig_path: self.config.tsconfig_path.as_deref(),
                editor_session: &self.editor_session,
            },
        )?;
        self.open_canon_project_documents(
            project.documents,
            project.session_project_root.clone(),
            project.session_config_path,
            project.materialized_changes,
        )
        .await?;
        Ok(CorsaScriptVirtualProject {
            document: project.host,
            materialized_sources: project.materialized_sources,
            source_catalog: project.source_catalog,
            session_project_root: project.session_project_root,
        })
    }
}

#[cfg(test)]
pub(super) fn build_script_virtual_project(
    source_path: &Path,
    request_path: &str,
    code: &str,
    source_type: SourceType,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &[(PathBuf, &str)],
) -> (
    String,
    Vec<(String, String)>,
    Option<PathBuf>,
    crate::batch::virtual_project::MaterializedFileDelta,
) {
    let virtual_ts_options = crate::virtual_ts::VirtualTsOptions::default();
    let request = CorsaScriptVirtualDocumentRequest {
        source_path,
        request_path,
        code,
        source_type,
        options,
        overlays,
        virtual_ts_options: &virtual_ts_options,
    };
    let virtual_ts_options = request.virtual_ts_options;
    let project = build_script_virtual_project_with_package_routes(
        request,
        None,
        super::vue_document::CorsaProjectEnvironment {
            virtual_ts_options,
            package_routes: &crate::PackageRouteResolver::default(),
            project_root: None,
            tsconfig_path: None,
            editor_session: super::editor_session::fallback_editor_session(),
        },
    )
    .expect("script virtual project");
    (
        project.host.request_uri,
        project.documents,
        project.session_project_root,
        project.materialized_changes,
    )
}

fn build_script_virtual_project_with_package_routes(
    request: CorsaScriptVirtualDocumentRequest<'_>,
    snapshot: Option<&super::CorsaOverlaySnapshot>,
    environment: super::vue_document::CorsaProjectEnvironment<'_>,
) -> Result<BuiltScriptVirtualProject, CorsaBridgeError> {
    let rewriter = ImportRewriter::new();
    let canonical_overlays;
    let (overlays, overlay_fingerprint): (
        &dyn super::vue_document::OverlaySources,
        Option<&super::vue_document::OverlayFingerprint>,
    ) = if let Some(snapshot) = snapshot {
        (snapshot.sources(), Some(snapshot.fingerprint()))
    } else {
        canonical_overlays = request
            .overlays
            .iter()
            .map(|(path, content)| {
                let key = vize_carton::path::canonicalize_non_verbatim(path);
                (key, *content)
            })
            .collect::<FxHashMap<_, _>>();
        (&canonical_overlays, None)
    };
    let alias_context =
        super::vue_dependencies_alias::AliasContext::for_hosts_cached_with_overlay_fingerprint(
            request.source_path,
            request.code,
            overlays,
            overlay_fingerprint,
            &[],
            request.options,
            environment,
        )?;
    let (path, generated) = alias_context
        .editor_script_document(request.source_path)
        .ok_or_else(|| {
            CorsaBridgeError::CommunicationError(vize_carton::cstr!(
                "Canon did not retain script projection for {}",
                request.source_path.display()
            ))
        })?;
    let request_uri = crate::file_uri::path_to_file_uri(&path);
    let materialized_sources = alias_context.materialized_sources();
    let mappings = materialized_sources
        .iter()
        .find(|source| source.materialized_path == path)
        .map(|source| source.mapping.spans().to_vec())
        .unwrap_or_default();
    let mut documents = vec![(request_uri.clone(), generated.code.clone())];
    let resolved_dependencies = collect_script_dependency_documents(
        &mut documents,
        request.source_path,
        request.code,
        request.source_type,
        request.options,
        &rewriter,
        &alias_context,
        overlays,
    );
    let session_project_root = alias_context.mirror_project_root_for_source(request.source_path);
    super::vue_document::materialized_documents::append_materialized_documents(
        &mut documents,
        &materialized_sources,
        overlays,
        false,
    );
    Ok(BuiltScriptVirtualProject {
        host: CorsaScriptVirtualDocument {
            request_uri,
            code: generated.code,
            mappings,
            import_source_map: generated.source_map,
            resolved_dependencies,
        },
        documents,
        session_project_root,
        session_config_path: alias_context.mirror_project_config_path(),
        materialized_changes: alias_context.materialized_changes.clone(),
        materialized_sources,
        source_catalog: alias_context.source_catalog.clone(),
    })
}
