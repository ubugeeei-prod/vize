//! Canonical Corsa project synchronization for JSX/TSX editor requests.

use oxc_span::SourceType;
use vize_canon::{CorsaBridge, CorsaScriptVirtualDocumentRequest, CorsaVueVirtualDocumentOptions};

use super::service::JsxService;
use super::virtual_ts::JsxVirtualTs;
use crate::ide::IdeContext;

pub(super) async fn open_virtual_project(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<(JsxVirtualTs, vize_s0::String)> {
    let source_path = ctx.uri.to_file_path().ok()?;
    let cached_overlays = ctx.state.corsa_overlays();
    let overlays = cached_overlays
        .iter()
        .map(|(path, content)| (path.clone(), &**content))
        .collect::<Vec<_>>();
    let virtual_ts_options = ctx.state.virtual_ts_options();
    let document = bridge
        .open_script_virtual_document_with_vue_dependencies(CorsaScriptVirtualDocumentRequest {
            source_path: &source_path,
            request_path: &JsxService::request_path(ctx.uri),
            code: &ctx.content,
            source_type: SourceType::from_path(&source_path).ok()?,
            options: CorsaVueVirtualDocumentOptions {
                options_api: ctx.state.options_api_enabled(),
                legacy_vue2: ctx.state.legacy_vue2_enabled(),
                jsx_typecheck: true,
                experimental_patterned_template: ctx.state.patterned_template_enabled(),
                preserve_event_navigation: true,
                dialect: ctx.state.type_checker_vue_version(),
            },
            overlays: &overlays,
            virtual_ts_options: &virtual_ts_options,
        })
        .await
        .ok()?;
    Some((
        JsxVirtualTs {
            code: document.code.into(),
            mappings: document.mappings,
            import_source_map: document.import_source_map,
        },
        document.request_uri,
    ))
}
