use crate::script::ScriptCompileContext;

/// Emit the __expose() call to the output buffer.
pub(super) fn emit_expose(output: &mut vize_carton::Vec<u8>, ctx: &ScriptCompileContext) {
    if let Some(ref expose_macro) = ctx.macros.define_expose {
        // args contains the argument content (e.g., "{ foo, bar }")
        let args = expose_macro.args.trim();
        if args.is_empty() {
            output.extend_from_slice(b"  __expose();\n");
        } else {
            output.extend_from_slice(b"  __expose(");
            output.extend_from_slice(args.as_bytes());
            output.extend_from_slice(b");\n");
        }
    } else {
        // No defineExpose, but still need to call __expose() for Vue runtime
        output.extend_from_slice(b"  __expose();\n");
    }
}
