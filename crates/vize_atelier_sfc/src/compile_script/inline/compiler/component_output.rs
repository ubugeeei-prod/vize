use crate::script::ScriptCompileContext;

use super::super::super::TemplateParts;

pub(super) struct ComponentState {
    pub(super) has_options: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_component_definition(
    output: &mut vize_carton::Vec<u8>,
    ctx: &ScriptCompileContext,
    component_name: &str,
    is_ts: bool,
    is_vapor: bool,
    is_async: bool,
    needs_prop_type: bool,
    needs_vapor_setup_context: bool,
    has_default_export: bool,
    props_emits_buf: &[u8],
    model_props_emits_buf: &[u8],
    template: &TemplateParts<'_>,
    vapor_render_alias: Option<&str>,
) -> ComponentState {
    // Start export default
    output.push(b'\n');
    let has_options = ctx.macros.define_options.is_some();

    // Setup function - include destructured args based on macros used
    let has_emit = ctx.macros.define_emits.is_some();
    let has_emit_binding = ctx
        .macros
        .define_emits
        .as_ref()
        .is_some_and(|emits| emits.binding_name.is_some());
    let has_expose = ctx.macros.define_expose.is_some();

    if has_options {
        // Use Object.assign for defineOptions
        if is_vapor {
            output.extend_from_slice(
                b"export default /*@__PURE__*/_defineVaporComponent(Object.assign(",
            );
        } else {
            output.extend_from_slice(b"export default /*@__PURE__*/Object.assign(");
        }
        let options_args = ctx.macros.define_options.as_ref().unwrap().args.trim();
        output.extend_from_slice(options_args.as_bytes());
        output.extend_from_slice(b", {\n");
    } else if has_default_export {
        // Normal script has export default that was rewritten to __default__
        // Use Object.assign to merge with setup component
        if is_vapor {
            output.extend_from_slice(
                b"export default /*@__PURE__*/_defineVaporComponent(Object.assign(__default__, {\n",
            );
        } else {
            output.extend_from_slice(b"export default /*@__PURE__*/Object.assign(__default__, {\n");
        }
    } else if is_vapor {
        output.extend_from_slice(b"export default /*@__PURE__*/_defineVaporComponent({\n");
    } else if is_ts {
        // TypeScript: use _defineComponent with __PURE__ annotation
        output.extend_from_slice(b"export default /*@__PURE__*/_defineComponent({\n");
    } else {
        output.extend_from_slice(b"export default {\n");
    }
    output.extend_from_slice(b"  __name: '");
    output.extend_from_slice(component_name.as_bytes());
    output.extend_from_slice(b"',\n");

    // Output props and emits definitions
    output.extend_from_slice(props_emits_buf);
    output.extend_from_slice(model_props_emits_buf);
    if !template.render_fn.is_empty() {
        output.extend_from_slice(b"  ");
        output.extend_from_slice(template.render_fn_name.as_bytes());
        output.extend_from_slice(b": ");
        if let Some(alias) = vapor_render_alias {
            output.extend_from_slice(alias.as_bytes());
        } else {
            output.extend_from_slice(template.render_fn_name.as_bytes());
        }
        output.extend_from_slice(b",\n");
    }

    // Build setup function signature based on what macros are used
    let mut setup_args = Vec::new();
    if has_expose {
        setup_args.push("expose: __expose");
    }
    if has_emit || needs_vapor_setup_context {
        if has_emit_binding || needs_vapor_setup_context {
            setup_args.push("emit: __emit");
        } else {
            setup_args.push("emit: $emit");
        }
    }
    if needs_vapor_setup_context {
        setup_args.push("attrs: __attrs");
        setup_args.push("slots: __slots");
    }

    // Add `: any` type annotation to __props when there are typed props in TypeScript mode
    // but NOT when needs_prop_type (defineComponent infers the type from PropType<T>)
    let has_typed_props = is_ts
        && ctx
            .macros
            .define_props
            .as_ref()
            .is_some_and(|p| p.type_args.is_some() || !p.args.is_empty());
    let props_param = if has_typed_props && !needs_prop_type {
        "__props: any"
    } else {
        "__props"
    };

    let async_prefix = if is_async {
        "  async setup("
    } else {
        "  setup("
    };
    if setup_args.is_empty() {
        output.extend_from_slice(async_prefix.as_bytes());
        output.extend_from_slice(props_param.as_bytes());
        output.extend_from_slice(b") {\n");
    } else {
        output.extend_from_slice(async_prefix.as_bytes());
        output.extend_from_slice(props_param.as_bytes());
        output.extend_from_slice(b", { ");
        output.extend_from_slice(setup_args.join(", ").as_bytes());
        output.extend_from_slice(b" }) {\n");
    }

    // Always add a blank line after setup signature
    output.push(b'\n');

    // Add __temp/__restore declarations for async setup
    if is_async {
        if is_ts {
            output.extend_from_slice(b"let __temp: any, __restore: any\n\n");
        } else {
            output.extend_from_slice(b"let __temp, __restore\n\n");
        }
    }

    ComponentState { has_options }
}
