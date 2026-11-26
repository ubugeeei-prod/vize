//! Entries of a non-inline setup `__returned__` object.

use crate::{script::ScriptCompileContext, types::BindingType};

/// Append the `__returned__` entry for `name`.
///
/// A reassignable setup declaration (`let`/`var`) is returned as an accessor
/// pair, so template writes reach the script variable and script
/// reassignments reach the template. Every other binding never changes
/// identity and is returned by value.
pub(crate) fn push_returned_binding(output: &mut Vec<u8>, name: &str, ctx: &ScriptCompileContext) {
    if !matches!(ctx.bindings.bindings.get(name), Some(BindingType::SetupLet)) {
        output.extend_from_slice(name.as_bytes());
        return;
    }
    // The setter parameter must not shadow the binding it assigns.
    let parameter: &[u8] = if name == "v" { b"_v" } else { b"v" };
    let name = name.as_bytes();
    output.extend_from_slice(b"get ");
    output.extend_from_slice(name);
    output.extend_from_slice(b"() { return ");
    output.extend_from_slice(name);
    output.extend_from_slice(b" }, set ");
    output.extend_from_slice(name);
    output.push(b'(');
    output.extend_from_slice(parameter);
    output.extend_from_slice(b") { ");
    output.extend_from_slice(name);
    output.extend_from_slice(b" = ");
    output.extend_from_slice(parameter);
    output.extend_from_slice(b" }");
}

#[cfg(test)]
mod tests {
    use super::push_returned_binding;
    use crate::{script::ScriptCompileContext, types::BindingType};
    use vize_carton::String;

    fn entries(names: &[&str], ctx: &ScriptCompileContext) -> Vec<u8> {
        let mut output = Vec::new();
        for (index, name) in names.iter().enumerate() {
            if index > 0 {
                output.extend_from_slice(b", ");
            }
            push_returned_binding(&mut output, name, ctx);
        }
        output
    }

    #[test]
    fn reassignable_setup_declarations_are_returned_as_accessors() {
        let mut ctx = ScriptCompileContext::new("");
        for (name, kind) in [
            ("el", BindingType::SetupLet),
            ("v", BindingType::SetupLet),
            ("count", BindingType::SetupRef),
            ("handler", BindingType::SetupConst),
        ] {
            ctx.bindings.bindings.insert(String::from(name), kind);
        }
        assert_eq!(
            entries(&["el", "v", "count", "handler", "imported"], &ctx).as_slice(),
            b"get el() { return el }, set el(v) { el = v }, get v() { return v }, set v(_v) { v = _v }, count, handler, imported"
        );
    }
}
