use std::path::PathBuf;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeError, CorsaVueVirtualDocumentOptions};

use super::{CanonicalDependencyDocument, CanonicalMaterializedSource, CanonicalVirtualDocument};
use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

pub(crate) async fn open_canonical_virtual_document(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<CanonicalVirtualDocument> {
    open_canonical_virtual_document_strict(ctx, bridge)
        .await
        .ok()
        .flatten()
}

/// Open the canonical document while preserving bridge failures for strict
/// editor-feature callers. The lenient wrapper above retains the production
/// fallback contract used by hover, completion, and definition.
pub(crate) async fn open_canonical_virtual_document_strict(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Result<Option<CanonicalVirtualDocument>, CorsaBridgeError> {
    let cached_overlays = ctx.state.corsa_overlays();
    let overlays = cached_overlays
        .iter()
        .map(|(path, content)| (path.clone(), &**content))
        .collect::<Vec<_>>();
    open_canonical_virtual_document_with_overlays_strict(ctx, bridge, &overlays).await
}

pub(super) async fn open_canonical_virtual_document_with_overlays_strict(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    overlays: &[(PathBuf, &str)],
) -> Result<Option<CanonicalVirtualDocument>, CorsaBridgeError> {
    open_canonical_virtual_document_with_sources_strict(ctx, bridge, overlays, &[]).await
}

pub(super) async fn open_canonical_virtual_document_with_sources_strict(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    overlays: &[(PathBuf, &str)],
    requested_sources: &[(PathBuf, &str)],
) -> Result<Option<CanonicalVirtualDocument>, CorsaBridgeError> {
    if !ctx.uri.path().ends_with(".vue") || ctx.uri.path().ends_with(".art.vue") {
        return Ok(None);
    }

    let Some(source_path) = ctx.uri.to_file_path().ok() else {
        return Ok(None);
    };
    let virtual_ts_options = ctx.state.virtual_ts_options();
    let opened = bridge
        .open_vue_virtual_workspace_document(
            &source_path,
            &ctx.content,
            CorsaVueVirtualDocumentOptions {
                options_api: ctx.state.options_api_enabled(),
                legacy_vue2: ctx.state.legacy_vue2_enabled(),
                jsx_typecheck: ctx.state.jsx_typecheck_enabled(),
                experimental_patterned_template: ctx.state.patterned_template_enabled(),
                preserve_event_navigation: true,
                dialect: ctx.state.type_checker_vue_version(),
            },
            overlays,
            &virtual_ts_options,
            requested_sources,
        )
        .await?;

    let dependencies = opened
        .dependencies
        .into_iter()
        .filter_map(|dependency| {
            let source_uri = authored_uri(ctx, &dependency.source_path)?;
            Some(CanonicalDependencyDocument {
                source_uri,
                source: dependency.source,
                request_uri: dependency.request_uri,
                virtual_result: VirtualTsResult::from_projection(
                    dependency.code.to_string(),
                    dependency.mapping,
                    dependency.import_source_map,
                ),
            })
        })
        .collect();

    let materialized_sources = map_materialized_sources(ctx, opened.materialized_sources);

    Ok(Some(CanonicalVirtualDocument {
        source_uri: ctx.uri.clone(),
        request_uri: opened.request_uri,
        virtual_result: VirtualTsResult::from_projection(
            opened.code.to_string(),
            opened.mapping,
            opened.import_source_map,
        ),
        dependencies,
        materialized_sources,
        session_project_roots: opened.session_project_root.into_iter().collect(),
    }))
}

pub(super) fn map_materialized_sources(
    ctx: &IdeContext<'_>,
    sources: Vec<vize_canon::CorsaMaterializedSource>,
) -> Vec<CanonicalMaterializedSource> {
    sources
        .into_iter()
        .filter_map(|materialized| {
            let source_uri = authored_uri(ctx, &materialized.source_path)?;
            let request_uri = Url::from_file_path(&materialized.materialized_path)
                .ok()?
                .to_string();
            Some(CanonicalMaterializedSource {
                source_uri,
                source: materialized.source,
                request_uri: request_uri.into(),
                virtual_result: VirtualTsResult::from_projection(
                    materialized.code.to_string(),
                    materialized.mapping,
                    materialized.import_source_map,
                ),
                mapping_kind: materialized.mapping_kind,
            })
        })
        .collect()
}

fn authored_uri(ctx: &IdeContext<'_>, source_path: &std::path::Path) -> Option<Url> {
    let source_path = vize_s0::path::canonicalize_non_verbatim(source_path);
    ctx.state
        .documents
        .uris()
        .into_iter()
        .find(|uri| {
            uri.to_file_path()
                .ok()
                .is_some_and(|path| vize_s0::path::canonicalize_non_verbatim(&path) == source_path)
        })
        .or_else(|| {
            let root = ctx.state.get_workspace_root()?;
            let physical_root = vize_s0::path::canonicalize_non_verbatim(&root);
            let relative = source_path.strip_prefix(physical_root).ok()?;
            Url::from_file_path(root.join(relative)).ok()
        })
        .or_else(|| Url::from_file_path(source_path).ok())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn closed_sources_keep_the_workspace_uri_and_open_sources_keep_the_editor_uri() {
        let root = tempfile::tempdir().unwrap();
        let physical = root.path().join("physical");
        let logical = root.path().join("workspace");
        std::fs::create_dir(&physical).unwrap();
        std::os::unix::fs::symlink(&physical, &logical).unwrap();
        let path = physical.join("shared.ts");
        std::fs::write(&path, "export const value = 1").unwrap();
        let state = crate::server::ServerState::new();
        state.set_workspace_root(logical.clone());
        let uri = Url::from_file_path(logical.join("App.vue")).unwrap();
        let ctx = IdeContext::testing(&state, &uri, 0, "<template />".into());
        assert_eq!(
            authored_uri(&ctx, &path),
            Some(Url::from_file_path(logical.join("shared.ts")).unwrap())
        );
        let opened_uri = Url::from_file_path(&path).unwrap();
        state.documents.open(
            opened_uri.clone(),
            "export const value = 2".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(authored_uri(&ctx, &path), Some(opened_uri));
    }
}
