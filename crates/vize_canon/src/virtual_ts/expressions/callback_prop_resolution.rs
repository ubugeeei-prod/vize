//! Resolve a generic child's instantiated props before checking inline callbacks.

use super::component_props::{
    ComponentPropSource, collect_generated_class_bindings, is_checkable_prop,
    merged_class_binding_value,
};
use super::prop_sources::{append_prop_value, generated_prop_value};
use super::spread_reserved_props::rewrite_reserved_spread_references;
use crate::virtual_ts::helpers::to_safe_identifier_fragment;
use crate::virtual_ts::scope::is_inline_callback_prop;
use vize_carton::{FxHashSet, String, append, cstr};
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
    template_prop_names: &FxHashSet<String>,
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
        "{expr_indent}const {resolved_props} = (undefined as unknown as __VizePropsResolver<typeof {component_ref}>)({{\n",
    );

    append_props_object(
        ts,
        usage,
        template_prop_names,
        source_context,
        expr_indent.as_str(),
        false,
    );
    append!(*ts, "{expr_indent}}});\n");
    append!(
        *ts,
        "{expr_indent}const {selected_props} = (undefined as unknown as __VizePropsSelector<typeof {resolved_props}>)({{\n",
    );
    append_props_object(
        ts,
        usage,
        template_prop_names,
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
    template_prop_names: &FxHashSet<String>,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
    erase_callbacks: bool,
) {
    let class_bindings = collect_generated_class_bindings(usage, template_prop_names);
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
                template_prop_names,
                source_context,
            );
            ts.push_str(",\n");
        }
        let value = if merge_class_bindings && prop.name.as_str() == "class" {
            if emitted_merged_class {
                continue;
            }
            emitted_merged_class = true;
            merged_class_binding_value(&class_bindings)
        } else {
            generated_prop_value(prop, template_prop_names)
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
        append_prop_value(ts, value.as_str());
        ts.push_str(",\n");
    }
    for spread in spreads {
        append!(*ts, "{expr_indent}  ...");
        append_spread_value(
            ts,
            usage,
            spread.expression.as_str(),
            template_prop_names,
            source_context,
        );
        ts.push_str(",\n");
    }
}

fn append_spread_value(
    ts: &mut String,
    usage: &ComponentUsage,
    expression: &str,
    template_prop_names: &FxHashSet<String>,
    source_context: ComponentPropSource<'_>,
) {
    if let Some(rewritten) = rewrite_reserved_spread_references(
        expression,
        template_prop_names,
        source_context.scopes,
        usage.scope_id,
    ) {
        append_prop_value(ts, rewritten.code.as_str());
    } else {
        append_prop_value(ts, expression);
    }
}
