//! Vue 2 pipe-filter asset resolution.

use vize_s0::String;

use super::EmitCx;
use super::buf::Buf;
use super::js::asset_ident;

pub(super) fn emit_resolves(cx: &mut EmitCx<'_>, names: &[String]) {
    cx.buf.use_resolve_filter();
    for name in names {
        cx.buf.push("const ");
        cx.buf.push(asset_ident("filter", name.as_str()).as_str());
        cx.buf.push(" = ");
        cx.buf.push(Buf::resolve_filter_alias());
        cx.buf.push("(\"");
        cx.buf.push(name.as_str());
        cx.buf.push("\")");
        cx.buf.newline();
    }
}
