use std::path::PathBuf;

use tower_lsp::lsp_types::Url;
use vize_canon::{
    CorsaBridge, CorsaBridgeError, CorsaVueVirtualDocumentOptions, CorsaVueWorkspaceRequest,
};

use super::{CanonicalDependencyDocument, CanonicalMaterializedSource, CanonicalVirtualDocument};
use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

struct AuthoredUriResolver {
    open: vize_l0::FxHashMap<PathBuf, Url>,
    workspace: Option<(PathBuf, PathBuf)>,
}

impl AuthoredUriResolver {
    fn new(ctx: &IdeContext<'_>) -> Self {
        let open = ctx
            .state
            .documents
            .uris()
            .into_iter()
            .filter_map(|uri| {
                let path = uri.to_file_path().ok()?;
                Some((ctx.state.canonical_editor_path(&path), uri))
            })
            .collect();
        let workspace = ctx.state.get_workspace_root().map(|root| {
            let physical = ctx.state.canonical_editor_path(&root);
            (root, physical)
        });
        Self { open, workspace }
    }

    fn resolve(&self, ctx: &IdeContext<'_>, source_path: &std::path::Path) -> Option<Url> {
        let source_path = ctx.state.canonical_editor_path(source_path);
        self.open
            .get(&source_path)
            .cloned()
            .or_else(|| {
                let (logical_root, physical_root) = self.workspace.as_ref()?;
                let relative = source_path.strip_prefix(physical_root).ok()?;
                Url::from_file_path(logical_root.join(relative)).ok()
            })
            .or_else(|| Url::from_file_path(source_path).ok())
    }
}

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
        tracing::warn!(uri = %ctx.uri, "canonical document skipped: URI is not a file path");
        return Ok(None);
    };
    let virtual_ts_options = ctx.state.editor_virtual_ts_options().await;
    let options = CorsaVueVirtualDocumentOptions {
        options_api: ctx.state.options_api_enabled(),
        legacy_vue2: ctx.state.legacy_vue2_enabled(),
        jsx_typecheck: ctx.state.jsx_typecheck_enabled(),
        experimental_patterned_template: ctx.state.patterned_template_enabled(),
        preserve_event_navigation: true,
        dialect: ctx.state.type_checker_vue_version(),
    };
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
    let opened = bridge
        .open_vue_virtual_workspace_document(
            &source_path,
            &ctx.content,
            options,
            overlays,
            &virtual_ts_options,
            requested_sources,
        )
        .await?;
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
                    dependency.code.to_string(),
                    dependency.mapping,
                    dependency.import_source_map,
                ),
            })
        })
        .collect();

    let materialized_sources =
        map_materialized_sources_with_resolver(ctx, &authored_uris, opened.materialized_sources);

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
        source_catalogs: vec![opened.source_catalog],
    }))
}

pub(super) fn map_materialized_sources(
    ctx: &IdeContext<'_>,
    sources: Vec<vize_canon::CorsaMaterializedSource>,
) -> Vec<CanonicalMaterializedSource> {
    let authored_uris = AuthoredUriResolver::new(ctx);
    map_materialized_sources_with_resolver(ctx, &authored_uris, sources)
}

fn map_materialized_sources_with_resolver(
    ctx: &IdeContext<'_>,
    authored_uris: &AuthoredUriResolver,
    sources: Vec<vize_canon::CorsaMaterializedSource>,
) -> Vec<CanonicalMaterializedSource> {
    sources
        .into_iter()
        .filter_map(|materialized| {
            let source_uri = authored_uris.resolve(ctx, &materialized.source_path)?;
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
