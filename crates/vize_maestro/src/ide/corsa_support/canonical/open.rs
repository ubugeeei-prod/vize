use std::path::PathBuf;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeError, CorsaVueWorkspaceRequest};

use super::super::AuthoredUriResolver;
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

/// Synchronize and borrow the immutable local projection needed by hover.
/// Cold requests with an admitted initial project keep the general path so
/// their direct startup receipt is still consumed exactly once.
pub(crate) async fn open_canonical_hover_document_strict(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Result<Option<vize_canon::CorsaVueHoverDocument>, CorsaBridgeError> {
    if !ctx.uri.path().ends_with(".vue") || ctx.uri.path().ends_with(".art.vue") {
        return Ok(None);
    }
    let Some(source_path) = ctx.uri.to_file_path().ok() else {
        tracing::warn!(uri = %ctx.uri, "canonical hover skipped: URI is not a file path");
        return Ok(None);
    };
    let cached_overlays = ctx.state.corsa_overlays();
    let overlays = cached_overlays
        .iter()
        .map(|(path, content)| (path.clone(), &**content))
        .collect::<Vec<_>>();
    let virtual_ts_options = ctx.state.editor_virtual_ts_options().await;
    let opened = bridge
        .open_vue_hover_document_with_borrowed_overlays_and_options(
            &source_path,
            &ctx.content,
            ctx.state.canonical_vue_document_options(),
            &overlays,
            &virtual_ts_options,
        )
        .await?;
    ctx.record_foreground_preparation();
    Ok(Some(opened))
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
        tracing::warn!(uri = %ctx.uri, "canonical document skipped: URI is not a file path");
        return Ok(None);
    };
    let virtual_ts_options = ctx.state.editor_virtual_ts_options().await;
    let options = ctx.state.canonical_vue_document_options();
    // Projecting a whole workspace takes long enough to starve every other
    // native request, so the transaction owner builds it between attempts.
    if !requested_sources.is_empty()
        && ctx.state.workspace_preparation.is_armed()
        && !bridge.vue_virtual_workspace_is_prepared(
            &source_path,
            &ctx.content,
            options,
            overlays,
            &virtual_ts_options,
            requested_sources,
        )
    {
        let preparation = bridge.prepare_vue_virtual_workspace(CorsaVueWorkspaceRequest {
            source_path: source_path.clone(),
            content: ctx.content.as_str().into(),
            options,
            overlays: overlays
                .iter()
                .map(|(path, content)| (path.clone(), (*content).into()))
                .collect(),
            virtual_ts_options: virtual_ts_options.clone(),
            requested_sources: requested_sources
                .iter()
                .map(|(path, content)| (path.clone(), (*content).into()))
                .collect(),
        });
        ctx.state
            .workspace_preparation
            .defer(Box::pin(preparation))
            .await;
    }
    let initial = requested_sources
        .is_empty()
        .then(|| ctx.take_initial_canonical_document(bridge))
        .flatten();
    let opened = if let Some(initial) = initial {
        // Canon still builds and validates the ordinary complete current graph.
        bridge
            .open_initial_vue_project(
                initial,
                &source_path,
                &ctx.content,
                options,
                overlays,
                &virtual_ts_options,
                requested_sources,
            )
            .await?
    } else {
        bridge
            .open_vue_virtual_workspace_document(
                &source_path,
                &ctx.content,
                options,
                overlays,
                &virtual_ts_options,
                requested_sources,
            )
            .await?
    };
    #[cfg(test)]
    ctx.state
        .record_editor_project_open("canonical", ctx.uri, &ctx.content, &opened);

    let authored_uris = AuthoredUriResolver::new(ctx);
    let dependencies = opened
        .dependencies
        .into_iter()
        .filter_map(|dependency| {
            let source_uri = authored_uris.resolve(ctx, &dependency.source_path)?;
            Some(CanonicalDependencyDocument {
                source_uri,
                source: dependency.source,
                request_uri: dependency.request_uri,
                virtual_result: VirtualTsResult::from_projection(
                    dependency.code,
                    dependency.mapping,
                    dependency.import_source_map,
                ),
            })
        })
        .collect();

    let materialized_sources =
        map_materialized_sources_with_resolver(ctx, &authored_uris, opened.materialized_sources);

    ctx.record_foreground_preparation();
    Ok(Some(CanonicalVirtualDocument {
        source_uri: ctx.uri.clone(),
        request_uri: opened.request_uri,
        virtual_result: VirtualTsResult::from_projection(
            opened.code,
            opened.mapping,
            opened.import_source_map,
        ),
        dependencies,
        materialized_sources,
        session_project_roots: opened.session_project_root.into_iter().collect(),
        source_catalogs: vec![opened.source_catalog],
    }))
}

pub(super) fn map_materialized_sources(
    ctx: &IdeContext<'_>,
    sources: Vec<MaterializedLeaf>,
) -> Vec<CanonicalMaterializedSource> {
    let authored_uris = AuthoredUriResolver::new(ctx);
    map_materialized_sources_with_resolver(ctx, &authored_uris, sources)
}

#[expect(
    clippy::disallowed_types,
    reason = "Canon shares materialized leaves with its source catalog"
)]
type MaterializedLeaf = std::sync::Arc<vize_canon::CorsaMaterializedSource>;

fn map_materialized_sources_with_resolver(
    ctx: &IdeContext<'_>,
    authored_uris: &AuthoredUriResolver,
    sources: Vec<MaterializedLeaf>,
) -> Vec<CanonicalMaterializedSource> {
    sources
        .iter()
        .filter_map(|materialized| {
            let source_uri = authored_uris.resolve(ctx, &materialized.source_path)?;
            let request_uri = Url::from_file_path(&materialized.materialized_path)
                .ok()?
                .to_string();
            Some(CanonicalMaterializedSource {
                source_uri,
                source: materialized.source.clone(),
                request_uri: request_uri.into(),
                virtual_result: VirtualTsResult::from_projection(
                    materialized.code.as_str().into(),
                    materialized.mapping.clone(),
                    materialized.import_source_map.clone(),
                ),
                mapping_kind: materialized.mapping_kind,
            })
        })
        .collect()
}

pub(super) fn authored_uri(ctx: &IdeContext<'_>, source_path: &std::path::Path) -> Option<Url> {
    AuthoredUriResolver::new(ctx).resolve(ctx, source_path)
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
