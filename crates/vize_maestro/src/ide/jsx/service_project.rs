//! Canonical Corsa project synchronization for JSX/TSX editor requests.

use oxc_span::SourceType;
use vize_canon::{CorsaBridge, CorsaVueVirtualDocumentOptions};

use super::service::JsxService;
use super::virtual_ts::JsxVirtualTs;
use crate::ide::IdeContext;

pub(super) async fn open_virtual_project(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    virtual_ts: &JsxVirtualTs,
) -> Option<vize_carton::String> {
    let source_path = ctx.uri.to_file_path().ok()?;
    let cached_overlays = ctx.state.corsa_overlays();
    let overlays = cached_overlays
        .iter()
        .map(|(path, content)| (path.clone(), &**content))
        .collect::<Vec<_>>();
    bridge
        .open_script_virtual_document_with_vue_dependencies(
            &source_path,
            &JsxService::request_path(ctx.uri),
            &virtual_ts.code,
            SourceType::ts(),
            CorsaVueVirtualDocumentOptions {
                options_api: ctx.state.options_api_enabled(),
                legacy_vue2: ctx.state.legacy_vue2_enabled(),
                preserve_event_navigation: true,
            },
            &overlays,
        )
        .await
        .ok()
}
