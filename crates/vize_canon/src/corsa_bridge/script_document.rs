//! Script-host synchronization for editor Corsa sessions.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, String};

use super::bridge::{CorsaBridge, normalize_document_uri};
use super::types::CorsaBridgeError;
use super::vue_dependencies::collect_script_dependency_documents;
use super::vue_document::CorsaVueVirtualDocumentOptions;
use crate::batch::ImportRewriter;

pub struct CorsaScriptVirtualDocumentRequest<'a> {
    pub source_path: &'a Path,
    pub request_path: &'a str,
    pub code: &'a str,
    pub source_type: SourceType,
    pub options: CorsaVueVirtualDocumentOptions,
    pub overlays: &'a [(PathBuf, &'a str)],
    pub virtual_ts_options: &'a crate::virtual_ts::VirtualTsOptions,
}

struct BuiltScriptVirtualProject {
    request_uri: String,
    documents: Vec<(String, String)>,
    session_project_root: Option<PathBuf>,
    materialized_changes: crate::batch::virtual_project::MaterializedFileDelta,
}

impl CorsaBridge {
    /// Open a generated TS/JS host together with every reachable Vue virtual
    /// document, using the same graph materialization as an SFC host.
    pub async fn open_script_virtual_document_with_vue_dependencies(
        &self,
        request: CorsaScriptVirtualDocumentRequest<'_>,
    ) -> Result<String, CorsaBridgeError> {
        let virtual_ts_options = request.virtual_ts_options;
        let project = build_script_virtual_project_with_package_routes(
            request,
            super::vue_document::CorsaProjectEnvironment {
                virtual_ts_options,
                package_routes: &self.package_route_resolver,
                project_root: self.config.working_dir.as_deref(),
                tsconfig_path: self.config.tsconfig_path.as_deref(),
                editor_session: &self.editor_session,
            },
        )?;
        self.open_canon_project_documents(
            &project.documents,
            project.session_project_root,
            project.materialized_changes,
        )
        .await?;
        Ok(project.request_uri)
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
        project.request_uri,
        project.documents,
        project.session_project_root,
        project.materialized_changes,
    )
}

fn build_script_virtual_project_with_package_routes(
    request: CorsaScriptVirtualDocumentRequest<'_>,
    environment: super::vue_document::CorsaProjectEnvironment<'_>,
) -> Result<BuiltScriptVirtualProject, CorsaBridgeError> {
    let rewriter = ImportRewriter::new();
    let overlays = request
        .overlays
        .iter()
        .map(|(path, content)| {
            let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            (key, *content)
        })
        .collect::<FxHashMap<_, _>>();
    let alias_context = super::vue_dependencies_alias::AliasContext::for_host_cached(
        request.source_path,
        request.code,
        &overlays,
        request.options,
        environment,
    )?;
    let request_uri = alias_context
        .mirror_virtual_path(request.source_path)
        .map(|path| crate::file_uri::path_to_file_uri(&path))
        .unwrap_or_else(|| normalize_document_uri(request.request_path));
    let mut documents = vec![(request_uri.clone(), request.code.into())];
    collect_script_dependency_documents(
        &mut documents,
        request.source_path,
        request.code,
        request.source_type,
        request.options,
        &rewriter,
        &alias_context,
        &overlays,
    );
    let session_project_root = alias_context.mirror_project_root_for_source(request.source_path);
    Ok(BuiltScriptVirtualProject {
        request_uri,
        documents,
        session_project_root,
        materialized_changes: alias_context.materialized_changes.clone(),
    })
}
