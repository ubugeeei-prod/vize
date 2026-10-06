//! Pass actual setup refs to the captured Vapor owner in every build mode.

use crate::{script::ScriptCompileContext, types::BindingType};
use vize_carton::String;

pub(super) fn emit_setter(
    output: &mut vize_carton::Vec<u8>,
    setup_bindings: &[String],
    ctx: &ScriptCompileContext,
) {
    let refs: Vec<_> = setup_bindings
        .iter()
        .filter(|name| {
            matches!(
                ctx.bindings.bindings.get(name.as_str()),
                Some(BindingType::SetupLet | BindingType::SetupRef | BindingType::SetupMaybeRef)
            )
        })
        .collect();
    if refs.is_empty() {
        output.extend_from_slice(b"const vaporTemplateRefSetter = _createTemplateRefSetter()\n");
        return;
    }
    // The runtime reads a string ref from setupState only in development.
    // Getters retain actual declarations (including reassignable setup lets),
    // rather than their proxy-unwrapped values or an instance-private field.
    output.extend_from_slice(b"const vaporTemplateRefSetter = ((bindings) => {\n");
    output.extend_from_slice(b"  const setRef = _createTemplateRefSetter()\n");
    output.extend_from_slice(b"  return (element, value, refFor, refKey) => {\n");
    output.extend_from_slice(b"    const bound = typeof value === 'string' && Object.prototype.hasOwnProperty.call(bindings, value)\n");
    output.extend_from_slice(b"    return setRef(element, bound ? bindings[value] : value, refFor, bound ? refKey ?? value : refKey)\n");
    output.extend_from_slice(b"  }\n})({ ");
    for (index, name) in refs.into_iter().enumerate() {
        if index > 0 {
            output.extend_from_slice(b", ");
        }
        output.extend_from_slice(b"get ");
        output.extend_from_slice(name.as_bytes());
        output.extend_from_slice(b"() { return ");
        output.extend_from_slice(name.as_bytes());
        output.extend_from_slice(b" }");
    }
    output.extend_from_slice(b" })\n");
}

#[cfg(test)]
mod tests {
    use super::emit_setter;
    use crate::{script::ScriptCompileContext, types::BindingType};
    use vize_carton::String;

    #[test]
    fn setter_joins_only_actual_ref_capable_setup_declarations() {
        let mut ctx = ScriptCompileContext::new("");
        let bindings: Vec<String> = ["el", "maybe", "mutable", "callback", "other"]
            .into_iter()
            .map(String::from)
            .collect();
        for (name, kind) in [
            ("el", BindingType::SetupRef),
            ("maybe", BindingType::SetupMaybeRef),
            ("mutable", BindingType::SetupLet),
            ("callback", BindingType::SetupConst),
        ] {
            ctx.bindings.bindings.insert(String::from(name), kind);
        }
        let mut output = vize_carton::Vec::new();
        emit_setter(&mut output, &bindings, &ctx);
        assert_eq!(
            output.as_slice(),
            b"const vaporTemplateRefSetter = ((bindings) => {\n  const setRef = _createTemplateRefSetter()\n  return (element, value, refFor, refKey) => {\n    const bound = typeof value === 'string' && Object.prototype.hasOwnProperty.call(bindings, value)\n    return setRef(element, bound ? bindings[value] : value, refFor, bound ? refKey ?? value : refKey)\n  }\n})({ get el() { return el }, get maybe() { return maybe }, get mutable() { return mutable } })\n"
        );
    }

    #[test]
    fn setter_without_ref_capable_bindings_keeps_the_original_factory() {
        let mut output = vize_carton::Vec::new();
        emit_setter(&mut output, &[], &ScriptCompileContext::new(""));
        assert_eq!(
            output.as_slice(),
            b"const vaporTemplateRefSetter = _createTemplateRefSetter()\n"
        );
    }
}
