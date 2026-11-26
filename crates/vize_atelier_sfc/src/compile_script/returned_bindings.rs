//! Entries of a non-inline setup `__returned__` object.

use vize_carton::string_builder::TextBuf;

use crate::{script::ScriptCompileContext, types::BindingType};

/// Append the `__returned__` entry for `name`.
///
/// A reassignable setup declaration (`let`/`var`) is returned as an accessor
/// pair, so template writes reach the script variable and script
/// reassignments reach the template. Every other binding never changes
/// identity and is returned by value.
pub(crate) fn push_returned_binding(output: &mut TextBuf, name: &str, ctx: &ScriptCompileContext) {
    if !matches!(ctx.bindings.bindings.get(name), Some(BindingType::SetupLet)) {
        output.push_str(name);
        return;
    }
    // The setter parameter must not shadow the binding it assigns.
    let parameter = if name == "v" { "_v" } else { "v" };
    for part in [
        "get ",
        name,
        "() { return ",
        name,
        " }, set ",
        name,
        "(",
        parameter,
        ") { ",
        name,
        " = ",
        parameter,
        " }",
    ] {
        output.push_str(part);
    }
}

#[cfg(test)]
mod tests {
    use super::push_returned_binding;
    use crate::{script::ScriptCompileContext, types::BindingType};
    use vize_carton::{String, string_builder::TextBuf};

    fn entries(names: &[&str], ctx: &ScriptCompileContext) -> TextBuf {
        let mut output = TextBuf::with_capacity(64);
        for (index, name) in names.iter().enumerate() {
            if index > 0 {
                output.push_str(", ");
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
            entries(&["el", "v", "count", "handler", "imported"], &ctx).as_str(),
            "get el() { return el }, set el(v) { el = v }, get v() { return v }, set v(_v) { v = _v }, count, handler, imported"
        );
    }
}
