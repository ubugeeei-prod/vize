//! Construct one exact editor query surface from a validated alias revision.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String};

use super::{
    CorsaBridgeError, CorsaProjectEnvironment, CorsaVueVirtualDocument,
    CorsaVueVirtualDocumentOptions, CorsaVueVirtualProject, ImportRewriter,
    collect_dependency_documents, generate_vue_document_with_options, materialized_documents,
    tsx_vue_import_shim,
};

pub(super) fn build_vue_virtual_workspace_project(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &[(PathBuf, &str)],
    requested_sources: &[(PathBuf, &str)],
    environment: CorsaProjectEnvironment<'_>,
) -> Result<CorsaVueVirtualProject, CorsaBridgeError> {
    let rewriter = ImportRewriter::new();
    let overlays = overlays
        .iter()
        .map(|(path, content)| {
            let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            (key, *content)
        })
        .collect::<FxHashMap<_, _>>();
    // The alias mirror is built before generation, and from the same buffers the
    // dependency walk reads, so a specifier the resolver rewrites always has a
    // materialized target (#3900).
    let alias_context =
        crate::corsa_bridge::vue_dependencies_alias::AliasContext::for_hosts_cached(
            source_path,
            content,
            &overlays,
            requested_sources,
            options,
            environment,
        )?;
    let host = generate_vue_document_with_options(
        source_path,
        content,
        options,
        environment.virtual_ts_options,
        &rewriter,
        Some(&alias_context),
    )?;
    let mut documents = vec![(host.virtual_uri.clone(), host.generated.code.clone())];
    let mut dependencies = Vec::new();
    if host.generated.virtual_suffix == ".tsx" {
        documents.push(tsx_vue_import_shim(&host.source_path, &host.virtual_uri));
    }
    let resolved_dependencies = collect_dependency_documents(
        &mut documents,
        &mut dependencies,
        &host,
        options,
        &rewriter,
        &alias_context,
        &overlays,
    );
    let generated = host.generated;
    let materialized_sources = alias_context.materialized_sources();
    if !requested_sources.is_empty() || !overlays.is_empty() {
        materialized_documents::append_materialized_documents(
            &mut documents,
            &materialized_sources,
            &overlays,
            !requested_sources.is_empty(),
        );
    }
    let session_project_root = alias_context.mirror_project_root_for_source(source_path);
    let materialized_changes = alias_context.materialized_changes.clone();
    Ok(CorsaVueVirtualProject {
        host: CorsaVueVirtualDocument {
            request_uri: host.virtual_uri,
            code: generated.code,
            pre_rewrite_code: generated.pre_rewrite_code,
            mapping: generated.mapping,
            import_source_map: generated.import_source_map,
            source_type: generated.source_type,
            virtual_suffix: generated.virtual_suffix,
            dependencies,
            resolved_dependencies,
            materialized_sources,
            source_catalog: alias_context.source_catalog.clone(),
            session_project_root: session_project_root.clone(),
        },
        documents,
        session_project_root,
        session_config_path: alias_context.mirror_project_config_path(),
        materialized_changes,
    })
}
