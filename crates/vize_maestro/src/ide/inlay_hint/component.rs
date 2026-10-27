//! A constant that holds an SFC shows the authored component contract rather
//! than the generated component machinery.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "tower_lsp::lsp_types payloads take std `String`"
)]

use tower_lsp::lsp_types::{InlayHint, InlayHintLabel};

use crate::ide::hover::{component_contract_inline, contains_generated_component_marker};
use crate::ide::{IdeContext, position_to_offset, token_span_at_offset};

pub(super) fn is_generated(label: &InlayHintLabel) -> bool {
    match label {
        InlayHintLabel::String(label) => contains_generated_component_marker(label),
        InlayHintLabel::LabelParts(parts) => contains_generated_component_marker(
            &parts
                .iter()
                .map(|part| part.value.as_str())
                .collect::<String>(),
        ),
    }
}

/// `None` drops a generated type that no authored SFC explains.
pub(super) fn authored_contract(ctx: &IdeContext<'_>, mut hint: InlayHint) -> Option<InlayHint> {
    let offset = position_to_offset(&ctx.content, hint.position.line, hint.position.character)?;
    let (start, end) = token_span_at_offset(&ctx.content, offset.checked_sub(1)?, |byte| {
        byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
    })?;
    if end != offset {
        return None;
    }
    let contract = component_contract_inline(ctx, ctx.content.get(start..end)?)?;
    hint.label = InlayHintLabel::String(format!(": {contract}"));
    // The generated type cannot be inserted as an annotation.
    hint.text_edits = None;
    hint.tooltip = None;
    Some(hint)
}
