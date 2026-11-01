//! Server-only value consumption of a genuine original Model opening.

use super::{SsrError, SsrErrorKind};
use crate::{
    runtime::{Runtime, vocabulary},
    write::{LinkSink, Writer},
};
use vize_l0::id::NodeId;
use vize_l2::op::ElementOp;
use vize_l3::decision::ssr::NativeSelectedSetupSsrAnalysis;

pub(super) fn write<'owner, 'arena, L: LinkSink>(
    writer: &mut Writer<L>,
    analysis: &NativeSelectedSetupSsrAnalysis<'_, 'owner, 'arena>,
    node: NodeId,
    element: &'owner ElementOp<'arena>,
) -> Result<bool, SsrError> {
    let fail = |kind| SsrError {
        node: Some(node),
        span: element.span,
        kind,
    };
    let Some(row) = analysis
        .model(node, element)
        .map_err(|reason| fail(SsrErrorKind::Unsupported(reason)))?
    else {
        return Ok(false);
    };
    let runtime = vocabulary(Runtime::VueServerRenderer);
    let attrs = runtime
        .helper("ssrRenderAttrs")
        .ok_or_else(|| fail(SsrErrorKind::MissingRuntimeHelper))?;
    let merge = runtime
        .helper("mergeProps")
        .ok_or_else(|| fail(SsrErrorKind::MissingRuntimeHelper))?;
    let dynamic_model = runtime
        .helper("ssrGetDynamicModelProps")
        .ok_or_else(|| fail(SsrErrorKind::MissingRuntimeHelper))?;
    writer.use_helper(attrs);
    writer.use_helper(merge);
    writer.use_helper(dynamic_model);
    // Root attrs may carry a real input type/value. The actual pinned runtime
    // chooses value/checked semantics before rendering the merged attributes.
    // `.trim` belongs to actual input/change handlers, never eager SSR values.
    writer.push("${_ssrRenderAttrs(_mergeProps(_attrs, _ssrGetDynamicModelProps(_attrs, ");
    writer.push_named_parts(
        ["$setup.", row.occurrence().name],
        row.authored_span(),
        row.occurrence().name,
    );
    writer.push(")))}");
    Ok(true)
}
