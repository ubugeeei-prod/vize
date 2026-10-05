//! Construct one exact editor query surface from a validated alias revision.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String};

use super::{
    CorsaBridgeError, CorsaProjectEnvironment, CorsaVueVirtualDocument,
    CorsaVueVirtualDocumentOptions, CorsaVueVirtualProject, ImportRewriter,
    collect_dependency_documents, generate_vue_document_with_options, materialized_documents,
    tsx_vue_import_shim,
};

/// Immutable query data belongs to the same bounded, self-validating alias revision.
/// Native session synchronization is intentionally performed on every open.
pub(in crate::corsa_bridge) struct QuerySurface {
    host: CorsaVueVirtualDocument,
    documents: Vec<(String, String)>,
    session_config_path: Option<PathBuf>,
}

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
    let surface = if let Some(surface) = alias_context.query_surface.get() {
        surface
    } else {
        let surface = build_surface(
            source_path,
            content,
            options,
            &overlays,
            requested_sources,
            environment,
            &rewriter,
            &alias_context,
        )?;
        // Concurrent preparation can share the winner because the validated
        // alias revision includes exact host/options/overlay/request membership.
        let _ = alias_context.query_surface.set(surface);
        alias_context
            .query_surface
            .get()
            .expect("prepared query surface")
    };
    let mut host = surface.host.clone();
    // Other hosts can extend the live catalog without replacing this context.
    // Retain the catalog captured by this exact preparation, not the first open.
    host.source_catalog = alias_context.source_catalog.clone();
    Ok(CorsaVueVirtualProject {
        session_project_root: host.session_project_root.clone(),
        host,
        documents: surface.documents.clone(),
        session_config_path: surface.session_config_path.clone(),
        materialized_changes: alias_context.materialized_changes.clone(),
    })
}

#[expect(clippy::too_many_arguments, reason = "exact query surface inputs")]
fn build_surface(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    overlays: &FxHashMap<PathBuf, &str>,
    requested_sources: &[(PathBuf, &str)],
    environment: CorsaProjectEnvironment<'_>,
    rewriter: &ImportRewriter,
    alias_context: &crate::corsa_bridge::vue_dependencies_alias::AliasContext,
) -> Result<QuerySurface, CorsaBridgeError> {
    let host = generate_vue_document_with_options(
        source_path,
        content,
        options,
        environment.virtual_ts_options,
        rewriter,
        Some(alias_context),
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
        rewriter,
        alias_context,
        overlays,
    );
    let generated = host.generated;
    let materialized_sources = alias_context.materialized_sources();
    if !requested_sources.is_empty() || !overlays.is_empty() {
        materialized_documents::append_materialized_documents(
            &mut documents,
            &materialized_sources,
            overlays,
            !requested_sources.is_empty(),
        );
    }
    let session_project_root = alias_context.mirror_project_root_for_source(source_path);
    Ok(QuerySurface {
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
            source_catalog: Default::default(),
            session_project_root: session_project_root.clone(),
        },
        documents,
        session_config_path: alias_context.mirror_project_config_path(),
    })
}

#[cfg(test)]
mod tests;
