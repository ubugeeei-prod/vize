use vize_s0::Span;

use crate::lower::TextPart;

use super::{EmitCx, EmitError, Reason, emit_dynamic_part, emit_quoted_text};
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
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
            cx.buf.newline();
        }
        cx.buf.use_create_text();
        cx.buf.push(Buf::create_text_alias());
        cx.buf.push("(");
        if part.dynamic {
            emit_dynamic_part(cx, part.text.as_str(), Site::SlotText)?;
            cx.buf.push(", 1 /* TEXT */");
        } else {
            emit_quoted_text(cx, part.text.as_str());
        }
        cx.buf.push(")");
    }
    Ok(())
}
