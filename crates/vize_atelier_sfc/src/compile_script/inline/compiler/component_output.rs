use crate::script::ScriptCompileContext;
use crate::types::{CssModuleMapping, css_modules_object_literal};

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
    css_modules: &[CssModuleMapping],
) -> ComponentState {
    // Start export default
    output.push(b'\n');
    let has_options = ctx.macros.define_options.is_some();

    // Setup function - include destructured args based on macros used
    let has_emit_binding = ctx
        .macros
        .define_emits
        .as_ref()
        .is_some_and(|emits| emits.binding_name.is_some());
    // The `emit` setup-context destructure is only injected when there is an emit
    // binding (`const emit = defineEmits(...)`) or when `$emit` is actually
    // referenced by the inline render fn. A bare `defineEmits([...])` whose `$emit`
    // is never used does not add an `emit` parameter (matches @vue/compiler-sfc).
    let has_emit = ctx.macros.define_emits.is_some()
        && (has_emit_binding || render_uses_dollar_emit(template));
    let has_expose = ctx.macros.define_expose.is_some();

    if let (true, Some(define_options)) = (has_options, ctx.macros.define_options.as_ref()) {
        let options_args = define_options.args.trim();
        if is_vapor {
            // Vapor wraps the merged options with Object.assign inside the helper.
            output.extend_from_slice(
                b"export default /*@__PURE__*/_defineVaporComponent(Object.assign(",
            );
            output.extend_from_slice(options_args.as_bytes());
            output.extend_from_slice(b", {\n");
        } else if is_ts {
            // TypeScript: spread the options into the _defineComponent call,
            // matching @vue/compiler-sfc (`_defineComponent({ ...{ … }, __name, … })`).
            output.extend_from_slice(b"export default /*@__PURE__*/_defineComponent({\n  ...");
            output.extend_from_slice(options_args.as_bytes());
            output.extend_from_slice(b",\n");
        } else {
            // JavaScript: Object.assign the options onto the setup component.
            output.extend_from_slice(b"export default /*@__PURE__*/Object.assign(");
            output.extend_from_slice(options_args.as_bytes());
            output.extend_from_slice(b", {\n");
        }
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
    if !css_modules.is_empty() {
        output.extend_from_slice(b"  __cssModules: ");
        output.extend_from_slice(css_modules_object_literal(css_modules, "  ").as_bytes());
        output.extend_from_slice(b",\n");
    }

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

/// Returns true if the inline render code references the `$emit` identifier.
fn render_uses_dollar_emit(template: &TemplateParts<'_>) -> bool {
    contains_identifier(template.render_body, "$emit")
        || contains_identifier(template.preamble, "$emit")
        || contains_identifier(template.render_fn, "$emit")
}

/// Substring search that requires the match to be a standalone identifier
/// (not part of a longer identifier such as `$emitFoo`). The `$` prefix already
/// guards the left boundary, so only the right boundary needs checking.
fn contains_identifier(haystack: &str, needle: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs = start + pos;
        let after = haystack[abs + needle.len()..].chars().next();
        if after.is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '$')) {
            return true;
        }
        start = abs + needle.len();
    }
    false
}
