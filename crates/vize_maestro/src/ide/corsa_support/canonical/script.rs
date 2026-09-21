//! Script-host projections retain the same cross-file mappings as Vue hosts.

use oxc_span::SourceType;
use vize_canon::{CorsaBridge, CorsaScriptVirtualDocumentRequest, CorsaVueVirtualDocumentOptions};

use super::CanonicalVirtualDocument;
use crate::ide::{IdeContext, diagnostics::VirtualTsResult};

pub(crate) async fn open_canonical_script_document(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    include_workspace: bool,
) -> Option<CanonicalVirtualDocument> {
    let source_path = ctx.uri.to_file_path().ok()?;
    let source_type = SourceType::from_path(&source_path).ok()?;
    if source_type.is_jsx() && !ctx.state.jsx_typecheck_enabled() {
        return None;
    }
    if include_workspace && let Some(document) = open_script_workspace(ctx, bridge).await {
        return Some(document);
    }
    let cached_overlays = ctx.state.corsa_overlays();
    let overlays = cached_overlays
        .iter()
        .map(|(path, content)| (path.clone(), &**content))
        .collect::<Vec<_>>();
    let virtual_ts_options = ctx.state.virtual_ts_options();
    let project = bridge
        .open_script_virtual_project(CorsaScriptVirtualDocumentRequest {
            source_path: &source_path,
            request_path: ctx.uri.as_str(),
            code: &ctx.content,
            source_type,
            options: CorsaVueVirtualDocumentOptions {
                options_api: ctx.state.options_api_enabled(),
                legacy_vue2: ctx.state.legacy_vue2_enabled(),
                jsx_typecheck: ctx.state.jsx_typecheck_enabled(),
                experimental_patterned_template: ctx.state.patterned_template_enabled(),
                preserve_event_navigation: true,
                dialect: ctx.state.type_checker_vue_version(),
            },
            overlays: &overlays,
            virtual_ts_options: &virtual_ts_options,
        })
        .await
        .ok()?;
    let mut opened = project.document;
    if !source_type.is_jsx() && opened.mappings.is_empty() {
        opened.mappings.push(vize_canon::virtual_ts::VizeMapping {
            src_range: 0..ctx.content.len(),
            gen_range: 0..ctx.content.len(),
            sub_spans: Vec::new(),
        });
    }
    Some(CanonicalVirtualDocument {
        source_uri: ctx.uri.clone(),
        request_uri: opened.request_uri,
        virtual_result: VirtualTsResult {
            code: opened.code.to_string(),
            source_mappings: opened.mappings,
            semantic_links: Vec::new(),
            import_source_map: opened.import_source_map,
        },
        dependencies: Vec::new(),
        materialized_sources: super::open::map_materialized_sources(
            ctx,
            project.materialized_sources,
        ),
        session_project_roots: project.session_project_root.into_iter().collect(),
    })
}

async fn open_script_workspace(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<CanonicalVirtualDocument> {
    let sources = super::project::same_typescript_project(
        ctx,
        ctx.state.discover_workspace_vue_sources().await,
    );
    let (uri, source) = sources
        .into_iter()
        .find(|(uri, _)| !uri.path().ends_with(".art.vue"))?;
    let host = IdeContext::with_content(ctx.state, &uri, 0, source);
    let mut document = super::open_canonical_virtual_workspace_document(&host, bridge).await?;
    let index = document
        .materialized_sources
        .iter()
        .position(|source| source.source_uri == *ctx.uri && source.mapping_kind.is_mappable())?;
    let mut selected = document.materialized_sources.remove(index);
    if !crate::utils::is_jsx_path(ctx.uri.path())
        && selected.virtual_result.source_mappings.is_empty()
    {
        selected
            .virtual_result
            .source_mappings
            .push(vize_canon::virtual_ts::VizeMapping {
                src_range: 0..ctx.content.len(),
                gen_range: 0..ctx.content.len(),
                sub_spans: Vec::new(),
            });
    }
    document
        .dependencies
        .push(super::CanonicalDependencyDocument {
            source_uri: std::mem::replace(&mut document.source_uri, ctx.uri.clone()),
            source: host.content.into(),
            request_uri: std::mem::replace(&mut document.request_uri, selected.request_uri),
            virtual_result: std::mem::replace(
                &mut document.virtual_result,
                selected.virtual_result,
            ),
        });
    Some(document)
}
