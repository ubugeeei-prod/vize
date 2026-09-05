//! Dynamic prop name array emission shared by element and component calls.

use vize_s0::String;

use super::EmitCx;

pub(super) fn emit_dynamic_props(cx: &mut EmitCx<'_>, names: &[String]) {
    if names.is_empty() {
        return;
    }
    cx.buf.push(", [");
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            cx.buf.push(", ");
        }
        cx.buf.push("\"");
        cx.buf.push(name.as_str());
        cx.buf.push("\"");
    }
    cx.buf.push("]");
}
