//! Resolve a generic child's instantiated props before checking inline callbacks.

use super::component_prop_bindings::{
    collect_generated_class_bindings, merged_class_binding_value,
};
use super::component_props::{ComponentPropSource, is_checkable_prop};
use super::prop_sources::{append_prop_value, generated_prop_value};
use super::spread_reserved_props::rewrite_reserved_spread_references;
use super::ts_suppression_comments::inference_callback_parameters_on_ignored_lines;
use crate::virtual_ts::helpers::to_safe_identifier_fragment;
use crate::virtual_ts::scope::is_inline_callback_prop;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::{String, cstr};
use vize_croquis::croquis::ComponentUsage;

pub(super) struct CallbackPropsResolution {
    pub(super) resolved_props: String,
    pub(super) selected_props: String,
}

pub(super) fn append_prop_check_type(
    ts: &mut crate::virtual_ts::TextBuf,
    (resolution, hoist_shared_preamble): (Option<&CallbackPropsResolution>, bool),
    (component_type_name, idx, safe_prop_name, prop_name): (&str, usize, &str, &str),
) {
    let Some(resolution) = resolution else {
        vize_l0::emit_ts!(
            ts,
            ": __{component_type_name}_{idx}_prop_{safe_prop_name} = "
        );
        return;
    };
    let camel_prop_name = super::super::helpers::to_camel_case(prop_name);
    let resolved_props = resolution.resolved_props.as_str();
    let selected_props = resolution.selected_props.as_str();
    if hoist_shared_preamble {
        let resolved_prop = crate::virtual_ts::helpers::shared_types::RESOLVED_PROP.select(true);
        vize_l0::emit_ts!(
            ts,
            ": {resolved_prop}<typeof {resolved_props}, typeof {selected_props}, '{camel_prop_name}', __{component_type_name}_{idx}_prop_{safe_prop_name}> = "
        );
    } else {
        vize_l0::emit_ts!(
            ts,
            ": $ResolvedProp<typeof {resolved_props}, typeof {selected_props}, '{camel_prop_name}', __{component_type_name}_{idx}_prop_{safe_prop_name}> = "
        );
    }
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
    ts: &mut crate::virtual_ts::TextBuf,
    usage: &ComponentUsage,
    idx: usize,
    component_ref: &str,
    template_binding_access: &TemplateBindingAccess,
    indent: &str,
    (source_context, hoist_shared_preamble): (ComponentPropSource<'_>, bool),
) -> Option<CallbackPropsResolution> {
    let scoped_access = template_binding_access.for_scope(usage.scope_id);
    let template_binding_access = &scoped_access;
    if !usage.props.iter().any(is_inline_callback_prop) {
        return None;
    }

    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
    let resolved_props = cstr!("__vize_resolved_{component_type_name}_props_{idx}");
    let selected_props = cstr!("__vize_selected_{component_type_name}_props_{idx}");
    let expr_indent = String::from(indent);
    if hoist_shared_preamble {
        let resolved_context_props =
            crate::virtual_ts::helpers::shared_types::RESOLVED_CONTEXT_PROPS.select(true);
        let props_resolver = crate::virtual_ts::helpers::shared_types::PROPS_RESOLVER.select(true);
        vize_l0::emit_ts!(
            ts,
            "{expr_indent}const {resolved_props} = (undefined as unknown as {resolved_context_props})((undefined as unknown as {props_resolver}<typeof {component_ref}>)({{\n"
        );
    } else {
        vize_l0::emit_ts!(
            ts,
            "{expr_indent}const {resolved_props} = (undefined as unknown as $ResolvedContextProps)((undefined as unknown as $PropsResolver<typeof {component_ref}>)({{\n"
        );
    }

    append_props_object(
        ts,
        usage,
        template_binding_access,
        source_context,
        expr_indent.as_str(),
        false,
    );
    vize_l0::emit_ts!(ts, "{expr_indent}}}));\n");
    if hoist_shared_preamble {
        let props_selector = crate::virtual_ts::helpers::shared_types::PROPS_SELECTOR.select(true);
        vize_l0::emit_ts!(
            ts,
            "{expr_indent}const {selected_props} = (undefined as unknown as {props_selector}<typeof {resolved_props}>)({{\n"
        );
    } else {
        vize_l0::emit_ts!(
            ts,
            "{expr_indent}const {selected_props} = (undefined as unknown as $PropsSelector<typeof {resolved_props}>)({{\n"
        );
    }
    append_props_object(
        ts,
        usage,
        template_binding_access,
        source_context,
        expr_indent.as_str(),
        true,
    );
    vize_l0::emit_ts!(ts, "{expr_indent}}});\n");
    Some(CallbackPropsResolution {
        resolved_props,
        selected_props,
    })
}

fn append_props_object(
    ts: &mut crate::virtual_ts::TextBuf,
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
            vize_l0::emit_ts!(ts, "{expr_indent}  ...");
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
            vize_l0::emit_ts!(
                ts,
                // Inference-only callback prop; mapped prop owner checks diagnostics.
                "{expr_indent}  // @ts-ignore\n"
            );
        }
        let name = super::super::helpers::to_camel_case(prop.name.as_str());
        vize_l0::emit_ts!(ts, "{expr_indent}  ");
        let key_start = ts.len();
        vize_l0::emit_ts!(ts, "\"{name}\"");
        let key_end = ts.len();
        ts.push_str(": ");
        // This copy is intentionally unmapped. `@ts-ignore` covers only the
        // next line, so the parameter tokens have to sit on the property line
        // even when the author broke them inside the opening parenthesis.
        let owned;
        let value = if inline_callback {
            let value = value.trim_start();
            owned = inference_callback_parameters_on_ignored_lines(value);
            owned.as_ref().map_or(value, |owned| owned.as_str())
        } else {
            value.as_str()
        };
        let value_range = append_prop_value(ts, value);
        source_context.inference_only().record_prop_projection(
            usage,
            prop,
            key_start..key_end,
            value_range,
        );
        ts.push_str(",\n");
    }
    for spread in spreads {
        vize_l0::emit_ts!(ts, "{expr_indent}  ...");
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

fn append_spread_value(
    ts: &mut crate::virtual_ts::TextBuf,
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
