//! Server-only value consumption of a genuine original Model opening.

use super::{SsrError, SsrErrorKind};
use crate::{
    runtime::{Runtime, vocabulary},
    write::{LinkSink, Writer},
};
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    file::{NativeFileModelKind, NativeFileModelState},
    op::ElementOp,
};
use vize_l3::decision::ssr::{NativeSelectedSetupSsrAnalysis, SsrFileModel};

pub(super) fn prelude<L: LinkSink>(
    writer: &mut Writer<L>,
    analysis: &NativeSelectedSetupSsrAnalysis<'_, '_, '_>,
) {
    let [original] = analysis.file().native_models() else {
        return;
    };
    if matches!(original.state(), NativeFileModelState::Admitted(_))
        && original.kind() == Some(NativeFileModelKind::Checkbox)
    {
        // This generated render-local never substitutes for an original binding.
        writer.push("let __vizeModelProps;");
        writer.newline();
    }
}

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
    match row.original().kind() {
        Some(NativeFileModelKind::Checkbox) => {
            let value = analysis
                .file()
                .native_attribute_value_for(0, element, 0)
                .ok_or_else(|| fail(SsrErrorKind::MissingAnalysis))?;
            let observation = value
                .observation()
                .ok_or_else(|| fail(SsrErrorKind::MissingAnalysis))?;
            let contain = runtime
                .helper("ssrLooseContain")
                .ok_or_else(|| fail(SsrErrorKind::MissingRuntimeHelper))?;
            writer.use_helper(contain);
            writer.push("${_ssrRenderAttrs((__vizeModelProps = _mergeProps({ ");
            let model_span = row
                .original()
                .input()
                .attribute_span()
                .ok_or_else(|| fail(SsrErrorKind::MissingAnalysis))?;
            let model_first = model_span.start < observation.name_span().start;
            if !model_first {
                checkbox_type(writer, observation.name_span(), observation.source().span());
                writer.push(", ");
            }
            writer.push("checked: Array.isArray(");
            access(writer, &row);
            writer.push(") ? _ssrLooseContain(");
            access(writer, &row);
            writer.push(", null) : ");
            access(writer, &row);
            if model_first {
                writer.push(", ");
                checkbox_type(writer, observation.name_span(), observation.source().span());
            }
            writer.push(" }, _attrs), _mergeProps(__vizeModelProps, _ssrGetDynamicModelProps(__vizeModelProps, ");
            access(writer, &row);
            writer.push("))))}");
            return Ok(true);
        }
        Some(NativeFileModelKind::Text) => {}
        None => return Err(fail(SsrErrorKind::MissingAnalysis)),
    }
    // Root attrs may carry a real input type/value. The actual pinned runtime
    // chooses value/checked semantics before rendering the merged attributes.
    // `.trim`, `.lazy` and `.number` are client handlers, never eager SSR values.
    writer.push("${_ssrRenderAttrs(_mergeProps(_attrs, _ssrGetDynamicModelProps(_attrs, ");
    access(writer, &row);
    writer.push(")))}");
    Ok(true)
}

// Mapping only: these two spans come from the already authenticated original
// value above. They cannot select or admit a Model kind.
fn checkbox_type<L: LinkSink>(writer: &mut Writer<L>, name: Span, value: Span) {
    writer.push_named("type", name, "type");
    writer.push(": \"");
    writer.push_named("checkbox", value, "checkbox");
    writer.push("\"");
}

fn access<L: LinkSink>(writer: &mut Writer<L>, row: &SsrFileModel<'_, '_>) {
    writer.push_named_parts(
        ["$setup.", row.occurrence().name],
        row.authored_span(),
        row.occurrence().name,
    );
}
