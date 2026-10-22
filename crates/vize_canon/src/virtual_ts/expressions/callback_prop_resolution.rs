//! Resolve a generic child's instantiated props before checking inline callbacks.

use super::component_props::{
    ComponentPropSource, collect_generated_class_bindings, is_checkable_prop,
    merged_class_binding_value,
};
use super::prop_sources::{append_prop_value, generated_prop_value};
use super::spread_reserved_props::rewrite_reserved_spread_references;
use crate::virtual_ts::helpers::to_safe_identifier_fragment;
use crate::virtual_ts::scope::is_inline_callback_prop;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::{String, append, cstr};
use vize_croquis::croquis::ComponentUsage;

pub(super) struct CallbackPropsResolution {
    pub(super) resolved_props: String,
    pub(super) selected_props: String,
}

/// Invoke the child's props resolver solely to capture its instantiated return
/// type, retaining inference from every authored prop and spread. This call has
/// no source mapping, so the mapped whole-props and per-prop checks below remain
/// the only user-facing diagnostic owners.
///
/// A second constrained identity call erases only the callbacks and captures
/// the authored sibling-prop literals. The mapped owner can therefore select a
/// discriminated-union branch after generic inference without letting an
/// invalid callback choose a different branch or emit a coarse duplicate.
pub(super) fn generate_callback_props_resolution(
    ts: &mut String,
    usage: &ComponentUsage,
    idx: usize,
    component_ref: &str,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    indent: &str,
) -> Option<CallbackPropsResolution> {
    if !usage.props.iter().any(is_inline_callback_prop) {
        return None;
    }

    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
    let resolved_props = cstr!("__vize_resolved_{component_type_name}_props_{idx}");
    let selected_props = cstr!("__vize_selected_{component_type_name}_props_{idx}");
    let expr_indent = String::from(indent);

    append!(
        *ts,
        "{expr_indent}const {resolved_props} = (undefined as unknown as __VizeResolvedContextProps)((undefined as unknown as __VizePropsResolver<typeof {component_ref}>)({{\n",
    );

    append_props_object(
        ts,
        usage,
        template_binding_access,
        source_context,
        expr_indent.as_str(),
        false,
    );
    append!(*ts, "{expr_indent}}}));\n");
    append!(
        *ts,
        "{expr_indent}const {selected_props} = (undefined as unknown as __VizePropsSelector<typeof {resolved_props}>)({{\n",
    );
    append_props_object(
        ts,
        usage,
        template_binding_access,
        source_context,
        expr_indent.as_str(),
        true,
    );
    append!(*ts, "{expr_indent}}});\n");
    Some(CallbackPropsResolution {
        resolved_props,
        selected_props,
    })
}

fn append_props_object(
    ts: &mut String,
    usage: &ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
    erase_callbacks: bool,
) {
    let class_bindings = collect_generated_class_bindings(usage, template_binding_access);
    let merge_class_bindings = class_bindings.len() > 1;
    let mut emitted_merged_class = false;
    let mut spreads = usage.spread_props.iter().peekable();
    for prop in &usage.props {
        if !is_checkable_prop(prop) {
            continue;
        }
        while let Some(spread) = spreads.next_if(|spread| spread.start < prop.start) {
            append!(*ts, "{expr_indent}  ...");
            append_spread_value(
                ts,
                usage,
                spread.expression.as_str(),
                template_binding_access,
                source_context,
            );
            ts.push_str(",\n");
        }
        let value = if merge_class_bindings && prop.name.as_str() == "class" {
            if emitted_merged_class {
                continue;
            }
            emitted_merged_class = true;
            merged_class_binding_value(&class_bindings, None)
        } else {
            generated_prop_value(prop, template_binding_access)
        };
        let Some(mut value) = value else { continue };
        let inline_callback = is_inline_callback_prop(prop);
        if erase_callbacks && inline_callback {
            value = String::from("undefined as any");
        } else if inline_callback {
            append!(
                *ts,
                "{expr_indent}  // @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n"
            );
        }
        let name = super::super::helpers::to_camel_case(prop.name.as_str());
        append!(*ts, "{expr_indent}  \"{name}\": ");
        // This copy is intentionally unmapped. `@ts-ignore` covers only the
        // next line, so the parameter tokens have to sit on the property line
        // even when the author broke them inside the opening parenthesis.
        let owned;
        let value = if inline_callback {
            owned = callback_parameters_on_first_line(value.trim_start());
            owned.as_str()
        } else {
            value.as_str()
        };
        append_prop_value(ts, value);
        ts.push_str(",\n");
    }
    for spread in spreads {
        append!(*ts, "{expr_indent}  ...");
        append_spread_value(
            ts,
            usage,
            spread.expression.as_str(),
            template_binding_access,
            source_context,
        );
        ts.push_str(",\n");
    }
}

/// Collapse line breaks that occur before the callback's own `=>`.
///
/// `(\n  params\n) => body` otherwise leaves `params` on a line `@ts-ignore`
/// does not cover, and the unmapped inference copy reports TS7006.
fn callback_parameters_on_first_line(value: &str) -> String {
    let Some(arrow) = arrow_token_offset(value) else {
        return String::from(value);
    };
    if !value.as_bytes()[..arrow].contains(&b'\n') && !value.as_bytes()[..arrow].contains(&b'\r') {
        return String::from(value);
    }
    let mut collapsed = String::with_capacity(value.len());
    for ch in value[..arrow].chars() {
        collapsed.push(if ch == '\n' || ch == '\r' { ' ' } else { ch });
    }
    collapsed.push_str(&value[arrow..]);
    collapsed
}

fn arrow_token_offset(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        match bytes[index] {
            b'/' if bytes[index + 1] == b'/' => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes[index + 1] == b'*' => {
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/')
                {
                    index += 1;
                }
                index = (index + 2).min(bytes.len());
            }
            b'\'' | b'"' | b'`' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() && bytes[index] != quote {
                    index += if bytes[index] == b'\\' { 2 } else { 1 };
                }
                index = (index + 1).min(bytes.len());
            }
            b'=' if bytes[index + 1] == b'>' => return Some(index),
            _ => index += 1,
        }
    }
    None
}

fn append_spread_value(
    ts: &mut String,
    usage: &ComponentUsage,
    expression: &str,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
) {
    if let Some(rewritten) = rewrite_reserved_spread_references(
        expression,
        template_binding_access,
        source_context.scopes,
        usage.scope_id,
    ) {
        append_prop_value(ts, rewritten.code.as_str());
    } else {
        append_prop_value(ts, expression);
    }
}
