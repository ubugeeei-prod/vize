//! Pass actual setup refs to the captured Vapor owner in every build mode.

use crate::{script::ScriptCompileContext, types::BindingType};
use vize_carton::String;
use vize_carton::string_builder::TextBuf;

pub(super) fn emit_setter(
    output: &mut TextBuf,
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
        output.push_str("const vaporTemplateRefSetter = _createTemplateRefSetter()\n");
        return;
    }
    // The runtime reads a string ref from setupState only in development.
    // Getters retain actual declarations (including reassignable setup lets),
    // rather than their proxy-unwrapped values or an instance-private field.
    output.push_str("const vaporTemplateRefSetter = ((bindings) => {\n");
    output.push_str("  const setRef = _createTemplateRefSetter()\n");
    output.push_str("  return (element, value, refFor, refKey) => {\n");
    output.push_str("    const bound = typeof value === 'string' && value in bindings\n");
    output.push_str("    return setRef(element, bound ? bindings[value] : value, refFor, bound ? refKey ?? value : refKey)\n");
    output.push_str("  }\n})({ __proto__: null, ");
    for (index, name) in refs.into_iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str("get ");
        output.push_str(name);
        output.push_str("() { return ");
        output.push_str(name);
        output.push_str(" }");
    }
    output.push_str(" })\n");
}

#[cfg(test)]
mod tests {
    use super::emit_setter;
    use crate::{script::ScriptCompileContext, types::BindingType};
    use vize_carton::String;
    use vize_carton::string_builder::TextBuf;

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
        let mut output = TextBuf::new();
        emit_setter(&mut output, &bindings, &ctx);
        assert_eq!(
            output.as_str(),
            "const vaporTemplateRefSetter = ((bindings) => {\n  const setRef = _createTemplateRefSetter()\n  return (element, value, refFor, refKey) => {\n    const bound = typeof value === 'string' && value in bindings\n    return setRef(element, bound ? bindings[value] : value, refFor, bound ? refKey ?? value : refKey)\n  }\n})({ __proto__: null, get el() { return el }, get maybe() { return maybe }, get mutable() { return mutable } })\n"
        );
    }

    #[test]
    fn setter_without_ref_capable_bindings_keeps_the_original_factory() {
        let mut output = TextBuf::new();
        emit_setter(&mut output, &[], &ScriptCompileContext::new(""));
        assert_eq!(
            output.as_str(),
            "const vaporTemplateRefSetter = _createTemplateRefSetter()\n"
        );
    }
}
