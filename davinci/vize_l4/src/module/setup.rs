//! Copy one sealed original JS setup body into the prepared component fragment.

use crate::write::{LinkSink, Writer};
use vize_l0::Span;
use vize_l2::lang::js::VueSetup;

pub const COMPONENT_BINDING: &str = "_sfc_main";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupEmitErrorKind {
    MissingBinding,
    GeneratedBindingCollision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupEmitError {
    pub span: Span,
    pub kind: SetupEmitErrorKind,
}

/// Copy the original admitted script exactly once, retaining full-file links.
/// Only genuine root setup bindings are returned. Mutable lexical bindings use
/// getters/setters; no caller-supplied setup state or initializer lowering exists.
/// Generated-name refusals happen before a writer or partial fragment is created.
pub fn emit_setup<L: LinkSink>(
    setup: &VueSetup<'_, '_, '_, '_>,
) -> Result<Writer<L>, SetupEmitError> {
    let fail = |span, kind| SetupEmitError { span, kind };
    for binding in setup.bindings() {
        let declaration = binding
            .declaration()
            .ok_or_else(|| fail(setup.source().span(), SetupEmitErrorKind::MissingBinding))?;
        if matches!(declaration.name.as_str(), "Object" | "__value" | "__v_raw") {
            return Err(fail(
                declaration.span,
                SetupEmitErrorKind::GeneratedBindingCollision,
            ));
        }
    }
    let mut writer = Writer::default();
    writer.push("const _sfc_main = {\n  setup(__props, { expose: __expose }) {\n    __expose();\n");
    let source = setup.source();
    writer.push_linked(source.source(), source.span());
    // The copied source may end in a line comment or omit its last semicolon.
    // Generated code always starts a separate statement on its own line.
    writer.push("\n;\n    const __returned__ = {\n");
    for binding in setup.bindings() {
        let declaration = binding
            .declaration()
            .ok_or_else(|| fail(source.span(), SetupEmitErrorKind::MissingBinding))?;
        let name = declaration.name.as_str();
        let span = declaration.span;
        writer.push("      get ");
        writer.push_named(name, span, name);
        writer.push("() { return ");
        writer.push_named(name, span, name);
        writer.push(" },\n      set ");
        writer.push_named(name, span, name);
        writer.push("(__value) { ");
        writer.push_named(name, span, name);
        writer.push(" = __value },\n");
    }
    writer.push("    };\n    Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true });\n    return __returned__;\n  }\n}\n");
    Ok(writer)
}
