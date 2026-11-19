//! Vue 2 pipe-filter asset resolution.

use vize_l0::String;

use super::EmitCx;
use super::buf::Buf;
use vize_l2::op::vue::asset_identifier;

pub(super) fn emit_resolves(cx: &mut EmitCx<'_>, names: &[String]) {
    cx.buf.use_resolve_filter();
    for name in names {
        cx.buf.push("const ");
        cx.buf
            .push(asset_identifier("filter", name.as_str()).as_str());
        cx.buf.push(" = ");
        cx.buf.push(Buf::resolve_filter_alias());
        cx.buf.push("(\"");
        cx.buf.push(name.as_str());
        cx.buf.push("\")");
        cx.buf.newline();
    }
}
