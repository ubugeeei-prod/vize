use std::sync::Arc;

use tower_lsp::lsp_types::SignatureHelp;
use vize_canon::CorsaBridge;

use super::{position::source_offset_to_virtual_position, service::JsxService, service_project};
use crate::ide::{IdeContext, SignatureHelpService, signature_help::SignatureHelpStage};

pub(in crate::ide) async fn signature_help_traced(
    ctx: &IdeContext<'_>,
    corsa_bridge: Option<Arc<CorsaBridge>>,
    context: Option<serde_json::Value>,
) -> (Option<SignatureHelp>, Vec<SignatureHelpStage>) {
    let mut stages = Vec::new();
    let Some(bridge) = corsa_bridge else {
        return (None, stages);
    };
    if !bridge.is_initialized() {
        return (None, stages);
    }
    let Some(virtual_ts) = JsxService::virtual_ts(ctx) else {
        stages.push(SignatureHelpStage::VirtualOpenFailed {
            message: "JSX virtual TypeScript generation returned no document".into(),
        });
        return (None, stages);
    };
    let Some((line, character)) =
        source_offset_to_virtual_position(&virtual_ts.code, &virtual_ts.mappings, ctx.offset)
    else {
        stages.push(SignatureHelpStage::VirtualOpenFailed {
            message: "JSX cursor did not map into virtual TypeScript".into(),
        });
        return (None, stages);
    };
    let uri = match service_project::open_virtual_project(ctx, &bridge, &virtual_ts).await {
        Some(uri) => {
            stages.push(SignatureHelpStage::VirtualOpened);
            uri
        }
        None => {
            stages.push(SignatureHelpStage::VirtualOpenFailed {
                message: "failed to open JSX virtual project".into(),
            });
            return (None, stages);
        }
    };
    let help = match bridge
        .signature_help_with_context(&uri, line, character, context)
        .await
    {
        Ok(Some(help)) => {
            stages.push(SignatureHelpStage::RequestSome);
            help
        }
        Ok(None) => {
            stages.push(SignatureHelpStage::RequestNull);
            return (None, stages);
        }
        Err(error) => {
            stages.push(SignatureHelpStage::RequestFailed {
                message: error.to_string(),
            });
            return (None, stages);
        }
    };
    (
        Some(SignatureHelpService::convert_lsp_signature_help(help)),
        stages,
    )
}
