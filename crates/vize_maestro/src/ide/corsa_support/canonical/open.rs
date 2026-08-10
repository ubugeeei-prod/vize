use std::path::PathBuf;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeError, CorsaVueVirtualDocumentOptions};

use super::{CanonicalDependencyDocument, CanonicalVirtualDocument};
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
    if !ctx.uri.path().ends_with(".vue") || ctx.uri.path().ends_with(".art.vue") {
        return Ok(None);
    }

    let Some(source_path) = ctx.uri.to_file_path().ok() else {
        return Ok(None);
    };
    let opened = bridge
        .open_vue_virtual_document_with_borrowed_overlays_and_options(
            &source_path,
            &ctx.content,
            CorsaVueVirtualDocumentOptions {
                options_api: ctx.state.options_api_enabled(),
                legacy_vue2: ctx.state.legacy_vue2_enabled(),
                preserve_event_navigation: true,
            },
            overlays,
            &vize_canon::virtual_ts::VirtualTsOptions::default(),
        )
        .await?;

    let dependencies = opened
        .dependencies
        .into_iter()
        .filter_map(|dependency| {
            let source_uri = Url::from_file_path(&dependency.source_path).ok()?;
            Some(CanonicalDependencyDocument {
                source_uri,
                source: dependency.source,
                request_uri: dependency.request_uri,
                virtual_result: VirtualTsResult {
                    code: dependency.code.to_string(),
                    source_mappings: dependency.mappings,
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

    Ok(Some(CanonicalVirtualDocument {
        request_uri: opened.request_uri,
        virtual_result: VirtualTsResult {
            code: opened.code.to_string(),
            source_mappings: opened.mappings,
            import_source_map: opened.import_source_map,
            user_code_start_line: 0,
            sfc_script_start_line: 0,
            template_scope_start_line: 0,
            line_mappings: Vec::new(),
            skipped_import_lines: 0,
        },
        dependencies,
    }))
}
