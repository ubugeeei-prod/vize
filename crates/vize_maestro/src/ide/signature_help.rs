//! TypeScript signature help for authored Vue and JSX positions.

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

#[cfg(feature = "native")]
use std::sync::Arc;

#[cfg(feature = "native")]
use tower_lsp::lsp_types::{
    Documentation, MarkupContent, MarkupKind, ParameterInformation, ParameterLabel, SignatureHelp,
    SignatureInformation,
};
#[cfg(feature = "native")]
use vize_canon::{
    CorsaBridge, LspDocumentation, LspParameterInformation, LspParameterLabel, LspSignatureHelp,
    LspSignatureInformation,
};

#[cfg(feature = "native")]
use super::IdeContext;
#[cfg(feature = "native")]
use super::corsa_support;
#[cfg(feature = "native")]
use super::hover::HoverService;
#[cfg(feature = "native")]
use crate::virtual_code::{ArtCursorPosition, ArtVariantInfo, BlockType, VirtualDocument};

/// Signature-help service.
pub struct SignatureHelpService;

#[cfg(feature = "native")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::ide) enum SignatureHelpStage {
    VirtualOpened,
    VirtualOpenFailed { message: String },
    RequestSome,
    RequestNull,
    RequestFailed { message: String },
}

#[cfg(feature = "native")]
impl SignatureHelpService {
    /// Resolve signature help through the same canonical virtual TypeScript
    /// project used by hover, completion, definition, and diagnostics.
    pub async fn signature_help_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<SignatureHelp> {
        Self::signature_help_with_corsa_context(ctx, corsa_bridge, None).await
    }

    /// Resolve signature help while preserving the triggering LSP context.
    pub async fn signature_help_with_corsa_context(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
        context: Option<serde_json::Value>,
    ) -> Option<SignatureHelp> {
        Self::signature_help_with_corsa_context_traced(ctx, corsa_bridge, context, None).await
    }

    #[cfg(test)]
    pub(in crate::ide) async fn signature_help_with_corsa_traced(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> (Option<SignatureHelp>, Vec<SignatureHelpStage>) {
        let mut trace = Vec::new();
        let help = Self::signature_help_with_corsa_context_traced(
            ctx,
            corsa_bridge,
            None,
            Some(&mut trace),
        )
        .await;
        (help, trace)
    }

    async fn signature_help_with_corsa_context_traced(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
        context: Option<serde_json::Value>,
        trace: Option<&mut Vec<SignatureHelpStage>>,
    ) -> Option<SignatureHelp> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        match ctx.block_type? {
            BlockType::Template | BlockType::Script | BlockType::ScriptSetup
                if !ctx.uri.path().ends_with(".art.vue") =>
            {
                Self::signature_help_in_canonical_sfc(ctx, &bridge, context).await
            }
            BlockType::Script => {
                Self::signature_help_in_split_script(ctx, &bridge, false, context, trace).await
            }
            BlockType::ScriptSetup => {
                Self::signature_help_in_split_script(ctx, &bridge, true, context, trace).await
            }
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::signature_help_in_art_variant(ctx, &bridge, info, context, trace).await
            }
            BlockType::Template | BlockType::Style(_) | BlockType::Art(_) => None,
        }
    }

    async fn signature_help_in_canonical_sfc(
        ctx: &IdeContext<'_>,
        bridge: &Arc<CorsaBridge>,
        context: Option<serde_json::Value>,
    ) -> Option<SignatureHelp> {
        let doc = corsa_support::open_canonical_virtual_document(ctx, bridge).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&doc, ctx.offset)?;
        let help = bridge
            .signature_help_with_context(&doc.request_uri, line, character, context)
            .await
            .ok()??;
        Some(Self::convert_lsp_signature_help(help))
    }

    async fn signature_help_in_split_script(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        is_setup: bool,
        context: Option<serde_json::Value>,
        trace: Option<&mut Vec<SignatureHelpStage>>,
    ) -> Option<SignatureHelp> {
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let script = if is_setup {
            virtual_docs.script_setup.as_ref()
        } else {
            virtual_docs.script.as_ref()
        }?;
        let generated_offset = HoverService::sfc_to_virtual_ts_script_offset(ctx, ctx.offset)?;
        Self::signature_help_in_virtual_document(
            ctx,
            bridge,
            script,
            generated_offset,
            corsa_support::script_request_path(ctx.uri, is_setup),
            context,
            trace,
        )
        .await
    }

    async fn signature_help_in_art_variant(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        info: &ArtVariantInfo,
        context: Option<serde_json::Value>,
        trace: Option<&mut Vec<SignatureHelpStage>>,
    ) -> Option<SignatureHelp> {
        let template = ctx
            .virtual_docs
            .as_ref()?
            .art_template(info.variant_index)?;
        let generated_offset = template
            .source_map
            .to_generated_for(ctx.offset as u32, |features| features.signature_help)?
            as usize;
        Self::signature_help_in_virtual_document(
            ctx,
            bridge,
            template,
            generated_offset,
            corsa_support::art_template_request_path(ctx.uri, info.variant_index),
            context,
            trace,
        )
        .await
    }

    async fn signature_help_in_virtual_document(
        _ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        document: &VirtualDocument,
        generated_offset: usize,
        request_path: vize_s0::String,
        context: Option<serde_json::Value>,
        mut trace: Option<&mut Vec<SignatureHelpStage>>,
    ) -> Option<SignatureHelp> {
        let (line, character) = super::offset_to_position(&document.content, generated_offset);
        let uri = match bridge
            .open_or_update_virtual_document(&request_path, &document.content)
            .await
        {
            Ok(uri) => {
                record_signature_help_stage(&mut trace, || SignatureHelpStage::VirtualOpened);
                uri
            }
            Err(error) => {
                record_signature_help_stage(&mut trace, || SignatureHelpStage::VirtualOpenFailed {
                    message: error.to_string(),
                });
                return None;
            }
        };
        let help = match bridge
            .signature_help_with_context(&uri, line, character, context)
            .await
        {
            Ok(Some(help)) => {
                record_signature_help_stage(&mut trace, || SignatureHelpStage::RequestSome);
                help
            }
            Ok(None) => {
                record_signature_help_stage(&mut trace, || SignatureHelpStage::RequestNull);
                return None;
            }
            Err(error) => {
                record_signature_help_stage(&mut trace, || SignatureHelpStage::RequestFailed {
                    message: error.to_string(),
                });
                return None;
            }
        };
        Some(Self::convert_lsp_signature_help(help))
    }

    pub(in crate::ide) fn convert_lsp_signature_help(help: LspSignatureHelp) -> SignatureHelp {
        SignatureHelp {
            signatures: help
                .signatures
                .into_iter()
                .map(Self::convert_signature)
                .collect(),
            active_signature: help.active_signature,
            active_parameter: help.active_parameter,
        }
    }

    fn convert_signature(signature: LspSignatureInformation) -> SignatureInformation {
        SignatureInformation {
            label: signature.label,
            documentation: signature.documentation.map(Self::convert_documentation),
            parameters: signature.parameters.map(|parameters| {
                parameters
                    .into_iter()
                    .map(Self::convert_parameter)
                    .collect()
            }),
            active_parameter: signature.active_parameter,
        }
    }

    fn convert_parameter(parameter: LspParameterInformation) -> ParameterInformation {
        ParameterInformation {
            label: match parameter.label {
                LspParameterLabel::String(label) => ParameterLabel::Simple(label),
                LspParameterLabel::Offsets(offsets) => ParameterLabel::LabelOffsets(offsets),
            },
            documentation: parameter.documentation.map(Self::convert_documentation),
        }
    }

    fn convert_documentation(documentation: LspDocumentation) -> Documentation {
        match documentation {
            LspDocumentation::String(value) => Documentation::String(value),
            LspDocumentation::Markup(markup) => Documentation::MarkupContent(MarkupContent {
                kind: if markup.kind == "markdown" {
                    MarkupKind::Markdown
                } else {
                    MarkupKind::PlainText
                },
                value: markup.value,
            }),
        }
    }
}

#[cfg(feature = "native")]
fn record_signature_help_stage(
    trace: &mut Option<&mut Vec<SignatureHelpStage>>,
    stage: impl FnOnce() -> SignatureHelpStage,
) {
    if let Some(trace) = trace.as_mut() {
        trace.push(stage());
    }
}

#[cfg(all(test, feature = "native"))]
mod tests;
