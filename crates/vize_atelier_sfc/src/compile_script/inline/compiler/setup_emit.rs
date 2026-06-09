use std::borrow::Cow;

use vize_atelier_core::{
    BindingMetadata, ExpressionNode, Position, SimpleExpressionNode, SourceLocation,
    TransformContext, TransformOptions, process_expression,
};
use vize_carton::{Box as CoreBox, String, profile};

use crate::script::{ScriptCompileContext, gen_props_access_exp};

use super::await_transform::transform_await_expressions;

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn replace_props_alias_access(code: &str, local: &str, replacement: &str) -> String {
    let needle = {
        let mut needle = String::with_capacity(local.len() + 8);
        needle.push_str("__props.");
        needle.push_str(local);
        needle
    };

    let mut result = String::with_capacity(code.len());
    let mut cursor = 0;
    while let Some(rel_pos) = code[cursor..].find(needle.as_str()) {
        let start = cursor + rel_pos;
        let end = start + needle.len();
        let after_ok = code[end..]
            .chars()
            .next()
            .is_none_or(|c| !is_identifier_continue(c));

        result.push_str(&code[cursor..start]);
        if after_ok {
            result.push_str(replacement);
        } else {
            result.push_str(&code[start..end]);
        }
        cursor = end;
    }
    result.push_str(&code[cursor..]);
    result
}

fn rewrite_props_aliases(code: String, bindings: &BindingMetadata) -> String {
    if bindings.props_aliases.is_empty() {
        return code;
    }

    let mut rewritten = code;
    for (local, key) in &bindings.props_aliases {
        let replacement = gen_props_access_exp(key);
        rewritten = replace_props_alias_access(&rewritten, local, &replacement);
    }
    rewritten
}

fn transform_css_var_expression(
    ctx: &ScriptCompileContext,
    var_expr: &str,
    source_is_ts: bool,
) -> String {
    let allocator = vize_carton::Bump::new();
    let loc = SourceLocation::new(
        Position::new(0, 1, 1),
        Position::new(var_expr.len() as u32, 1, var_expr.len() as u32 + 1),
        var_expr,
    );
    let exp = ExpressionNode::Simple(CoreBox::new_in(
        SimpleExpressionNode::new(var_expr, false, loc),
        &allocator,
    ));
    let mut transform_ctx = TransformContext::new(
        &allocator,
        String::default(),
        TransformOptions {
            prefix_identifiers: true,
            inline: true,
            is_ts: source_is_ts,
            binding_metadata: Some(ctx.bindings.clone()),
            ..Default::default()
        },
    );

    let code = match process_expression(&mut transform_ctx, &exp, false) {
        ExpressionNode::Simple(simple) => simple.content.clone(),
        ExpressionNode::Compound(_) => String::new(var_expr),
    };

    rewrite_props_aliases(code, &ctx.bindings)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_setup_body(
    output: &mut vize_carton::Vec<u8>,
    ctx: &ScriptCompileContext,
    model_infos: &[(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )],
    setup_body_lines: &[String],
    source_is_ts: bool,
    _is_ts: bool,
    is_async: bool,
    css_vars: &[Cow<'_, str>],
    scope_id: &str,
    css_vars_id: &str,
    is_prod: bool,
    has_css_vars: bool,
    setup_css_module_names: &[String],
) {
    // Emit binding: const emit = __emit
    if let Some(ref emits_macro) = ctx.macros.define_emits
        && let Some(ref binding_name) = emits_macro.binding_name
    {
        output.extend_from_slice(b"const ");
        output.extend_from_slice(binding_name.as_bytes());
        output.extend_from_slice(b" = __emit\n");
        if has_blank_line_after_macro(ctx.source.as_str(), emits_macro.end) {
            output.push(b'\n');
        }
    }

    // Props binding: const props = __props
    if let Some(ref props_macro) = ctx.macros.define_props
        && let Some(ref binding_name) = props_macro.binding_name
    {
        output.extend_from_slice(b"const ");
        output.extend_from_slice(binding_name.as_bytes());
        output.extend_from_slice(b" = __props");
        if has_semicolon_after_macro(ctx.source.as_str(), props_macro.end) {
            output.push(b';');
        }
        output.push(b'\n');
    }

    // Model bindings: const model = _useModel<T>(__props, 'modelValue')
    if !model_infos.is_empty() {
        for (model_name, binding_name, modifiers_binding_name, _, type_arg) in model_infos {
            output.extend_from_slice(b"const ");
            if let Some(modifiers_binding_name) = modifiers_binding_name {
                output.push(b'[');
                output.extend_from_slice(binding_name.as_bytes());
                output.extend_from_slice(b", ");
                output.extend_from_slice(modifiers_binding_name.as_bytes());
                output.extend_from_slice(b"]");
            } else {
                output.extend_from_slice(binding_name.as_bytes());
            }
            output.extend_from_slice(b" = _useModel");
            // Thread an explicit `<T>` type argument through to `useModel` so the
            // model ref's type matches @vue/compiler-sfc in TypeScript output.
            if source_is_ts
                && let Some(type_arg) = type_arg.as_deref().map(str::trim)
                && !type_arg.is_empty()
            {
                output.push(b'<');
                output.extend_from_slice(type_arg.as_bytes());
                output.push(b'>');
            }
            output.extend_from_slice(b"(__props, \"");
            output.extend_from_slice(model_name.as_bytes());
            output.extend_from_slice(b"\")\n");
        }
    }

    // Slots binding: const slots = _useSlots()
    if let Some(ref slots_macro) = ctx.macros.define_slots
        && let Some(ref binding_name) = slots_macro.binding_name
    {
        output.extend_from_slice(b"const ");
        output.extend_from_slice(binding_name.as_bytes());
        output.extend_from_slice(b" = _useSlots()\n");
    }

    // useCssVars injection for v-bind() in <style>
    if has_css_vars {
        output.extend_from_slice(b"_useCssVars(_ctx => ({\n");
        for (i, var_expr) in css_vars.iter().enumerate() {
            let var_name = if is_prod {
                crate::css::prod_scoped_v_bind_name(css_vars_id, var_expr)
            } else {
                crate::css::scoped_v_bind_name(scope_id, var_expr)
            };
            let var_value = transform_css_var_expression(ctx, var_expr, source_is_ts);
            output.extend_from_slice(b"  \"");
            output.extend_from_slice(var_name.as_bytes());
            output.extend_from_slice(b"\": (");
            output.extend_from_slice(var_value.as_bytes());
            output.extend_from_slice(b")");
            if i < css_vars.len() - 1 {
                output.extend_from_slice(b",");
            }
            output.extend_from_slice(b"\n");
        }
        output.extend_from_slice(b"}))\n\n");
    }

    for module_name in setup_css_module_names {
        output.extend_from_slice(b"const ");
        output.extend_from_slice(module_name.as_bytes());
        output.extend_from_slice(b" = _useCssModule(");
        if module_name != "$style" {
            output.extend_from_slice(json_string(module_name).as_bytes());
        }
        output.extend_from_slice(b")\n");
    }
    if !setup_css_module_names.is_empty() {
        output.push(b'\n');
    }

    // Output setup code lines (non-hoisted), transforming await expressions for async setup
    if is_async {
        let transformed_async = profile!(
            "atelier.script_inline.transform_await",
            transform_await_expressions(setup_body_lines, source_is_ts)
        );
        for line in &transformed_async {
            output.extend_from_slice(line.as_bytes());
            output.push(b'\n');
        }
    } else {
        for line in setup_body_lines {
            output.extend_from_slice(line.as_bytes());
            output.push(b'\n');
        }
    }

    // defineExpose: transform to __expose(...)
    if let Some(ref expose_macro) = ctx.macros.define_expose {
        let args = expose_macro.args.trim();
        output.extend_from_slice(b"__expose(");
        output.extend_from_slice(args.as_bytes());
        output.extend_from_slice(b")\n");
    }
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value)
        .map(|value| String::from(value.as_str()))
        .unwrap_or_else(|_| {
            let mut escaped = String::with_capacity(value.len() + 2);
            escaped.push('"');
            escaped.push_str(value);
            escaped.push('"');
            escaped
        })
}

fn has_blank_line_after_macro(source: &str, macro_end: usize) -> bool {
    let Some(mut rest) = source.get(macro_end..) else {
        return false;
    };

    rest = rest.trim_start_matches([' ', '\t']);
    if let Some(after_semicolon) = rest.strip_prefix(';') {
        rest = after_semicolon.trim_start_matches([' ', '\t']);
    }

    let mut newline_count = 0usize;
    for byte in rest.bytes() {
        match byte {
            b'\n' => {
                newline_count += 1;
                if newline_count >= 2 {
                    return true;
                }
            }
            b'\r' | b' ' | b'\t' => {}
            _ => return false,
        }
    }

    false
}

fn has_semicolon_after_macro(source: &str, macro_end: usize) -> bool {
    let Some(rest) = source.get(macro_end..) else {
        return false;
    };

    rest.trim_start_matches([' ', '\t']).starts_with(';')
}

#[cfg(test)]
mod tests {
    use super::{has_blank_line_after_macro, has_semicolon_after_macro};

    #[test]
    fn detects_blank_line_after_macro_call() {
        let source = "const emit = defineEmits<{ change: [] }>();\n\nfunction call() {}";
        let macro_end = source.find(";\n").unwrap();

        assert!(has_blank_line_after_macro(source, macro_end));
    }

    #[test]
    fn ignores_single_newline_after_macro_call() {
        let source = "const emit = defineEmits(['change'])\nfunction call() {}";
        let macro_end = source.find('\n').unwrap();

        assert!(!has_blank_line_after_macro(source, macro_end));
    }

    #[test]
    fn ignores_non_whitespace_after_macro_call() {
        let source = "const emit = defineEmits(['change']); // comment\nfunction call() {}";
        let macro_end = source.find(';').unwrap();

        assert!(!has_blank_line_after_macro(source, macro_end));
    }

    #[test]
    fn detects_semicolon_after_macro_call() {
        let source = "const props = defineProps<Props>();\n";
        let macro_end = source.find(';').unwrap();

        assert!(has_semicolon_after_macro(source, macro_end));
    }
}
