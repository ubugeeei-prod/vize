use std::path::PathBuf;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeError, CorsaVueVirtualDocumentOptions};

use super::{CanonicalDependencyDocument, CanonicalMaterializedSource, CanonicalVirtualDocument};
use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;
use crate::ide::diagnostics::corsa::semantic_links_after_import_rewrite;

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
    if !ctx.uri.path().ends_with(".vue") || ctx.uri.path().ends_with(".art.vue") {
        return Ok(None);
    }

    let Some(source_path) = ctx.uri.to_file_path().ok() else {
        return Ok(None);
    };
    let virtual_ts_options = ctx.state.virtual_ts_options();
    let opened = bridge
        .open_vue_virtual_document_with_borrowed_overlays_and_options(
            &source_path,
            &ctx.content,
            CorsaVueVirtualDocumentOptions {
                options_api: ctx.state.options_api_enabled(),
                legacy_vue2: ctx.state.legacy_vue2_enabled(),
                preserve_event_navigation: true,
                dialect: ctx.state.type_checker_vue_version(),
            },
            overlays,
            &virtual_ts_options,
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
                virtual_result: VirtualTsResult {
                    code: dependency.code.to_string(),
                    source_mappings: dependency.mappings,
                    semantic_links: semantic_links_after_import_rewrite(
                        dependency.semantic_links,
                        &dependency.import_source_map,
                    ),
                    import_source_map: dependency.import_source_map,
                    user_code_start_line: 0,
                    sfc_script_start_line: 0,
                    template_scope_start_line: 0,
                    line_mappings: Vec::new(),
                    skipped_import_lines: 0,
                },
            })
        })
        .collect();

    let materialized_sources = opened
        .materialized_sources
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
                virtual_result: VirtualTsResult {
                    code: materialized.code.to_string(),
                    source_mappings: materialized.mappings,
                    semantic_links: semantic_links_after_import_rewrite(
                        materialized.semantic_links,
                        &materialized.import_source_map,
                    ),
                    import_source_map: materialized.import_source_map,
                    user_code_start_line: 0,
                    sfc_script_start_line: 0,
                    template_scope_start_line: 0,
                    line_mappings: Vec::new(),
                    skipped_import_lines: 0,
                },
                mapping_kind: materialized.mapping_kind,
            })
        })
        .collect();

    Ok(Some(CanonicalVirtualDocument {
        source_uri: ctx.uri.clone(),
        request_uri: opened.request_uri,
        virtual_result: VirtualTsResult {
            code: opened.code.to_string(),
            source_mappings: opened.mappings,
            semantic_links: semantic_links_after_import_rewrite(
                opened.semantic_links,
                &opened.import_source_map,
            ),
            import_source_map: opened.import_source_map,
            user_code_start_line: 0,
            sfc_script_start_line: 0,
            template_scope_start_line: 0,
            line_mappings: Vec::new(),
            skipped_import_lines: 0,
        },
        dependencies,
        materialized_sources,
        session_project_roots: opened.session_project_root.into_iter().collect(),
    }))
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
        .or_else(|| Url::from_file_path(source_path).ok())
}
