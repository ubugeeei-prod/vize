use crate::script::ScriptCompileContext;
use vize_carton::string_builder::TextBuf;

/// Emit the __expose() call to the output buffer.
pub(super) fn emit_expose(output: &mut TextBuf, ctx: &ScriptCompileContext) {
    if let Some(ref expose_macro) = ctx.macros.define_expose {
        // args contains the argument content (e.g., "{ foo, bar }")
        let args = expose_macro.args.trim();
        if args.is_empty() {
            output.push_str("  __expose();\n");
        } else {
            output.push_str("  __expose(");
            output.push_str(args);
            output.push_str(");\n");
        }
    } else {
        // No defineExpose, but still need to call __expose() for Vue runtime
        output.push_str("  __expose();\n");
    }
}
