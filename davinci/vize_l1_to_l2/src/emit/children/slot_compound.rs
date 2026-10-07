use vize_l0::Span;

use crate::lower::TextPart;

use super::{EmitCx, EmitError, Reason, emit_compound_parts};
use crate::emit::buf::Buf;
use crate::emit::prefix::Site;

pub(super) fn emit_slot_compound_parts(
    cx: &mut EmitCx<'_>,
    parts: &[TextPart],
    span: Span,
) -> Result<(), EmitError> {
    if parts.is_empty() {
        return Err(EmitError::unsupported_at(Reason::EmptyCompoundText, span));
    }
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    cx.buf.push("(");
    emit_compound_parts(cx, parts, span, Site::SlotText)?;
    if parts.iter().any(|part| part.dynamic) {
        cx.buf.push(", 1 /* TEXT */");
    }
    cx.buf.push(")");
    Ok(())
}
