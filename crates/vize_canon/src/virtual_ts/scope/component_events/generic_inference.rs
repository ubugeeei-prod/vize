use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, EventHandlerScopeData, Scope, analysis::ComponentUsage};

use crate::virtual_ts::{
    expressions::rewrite_reserved_template_prop, helpers::to_camel_case,
    scope::is_inline_callback_prop,
};

use crate::virtual_ts::expressions::generated_prop_value;

pub(super) struct EmitInferenceContext<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) component_name: &'a str,
    pub(super) data: &'a EventHandlerScopeData,
    pub(super) scope: &'a Scope,
    pub(super) component_ref: &'a str,
    pub(super) component_type_name: &'a str,
    pub(super) safe_event_name: &'a str,
    pub(super) prop_key: &'a str,
    pub(super) template_prop_names: &'a FxHashSet<String>,
    pub(super) indent: &'a str,
}

pub(super) fn generate_inferred_emit_args(
    ts: &mut String,
    ctx: &EmitInferenceContext<'_>,
) -> Option<String> {
    ctx.summary.binding_spans.get(ctx.component_name)?;
    let (usage_idx, usage) =
        find_component_usage_for_event(ctx.summary, ctx.component_name, ctx.data, ctx.scope)?;
    if !usage.props.iter().any(|prop| {
        !prop.name_is_dynamic
            && prop.name.as_str() != "key"
            && prop.name.as_str() != "ref"
            && prop.value.is_some()
            && prop.is_dynamic
    }) {
        return None;
    }

    let scope_id = ctx.scope.id.as_u32();
    let resolver_type = cstr!(
        "__{}_{}_{}_emit_resolver",
        ctx.component_type_name,
        scope_id,
        ctx.safe_event_name
    );
    let emit_props = cstr!(
        "__vize_emit_props_{}_{}_{}",
        usage_idx,
        scope_id,
        ctx.safe_event_name
    );
    let inferred_args = cstr!(
        "__{}_{}_{}_inferred_emit_args",
        ctx.component_type_name,
        scope_id,
        ctx.safe_event_name
    );
    append!(
        *ts,
        "{}type {resolver_type} = typeof {} extends {{ __vizeResolveEmitProps?: infer __F }} ? (__F extends (...args: any[]) => any ? __F : (props: any) => {{}}) : typeof {} extends {{ __vizeResolveProps?: infer __F }} ? (__F extends (...args: any[]) => any ? __F : (props: any) => {{}}) : (props: any) => {{}};\n",
        ctx.indent,
        ctx.component_ref,
        ctx.component_ref,
    );
    let guard = usage.vif_guard.as_ref().map(|guard| {
        rewrite_reserved_template_prop(guard.as_str(), ctx.template_prop_names)
            .unwrap_or_else(|| guard.clone())
    });
    let guarded_call_indent = guard.as_ref().map(|_| cstr!("{}  ", ctx.indent));
    let call_indent = guarded_call_indent.as_deref().unwrap_or(ctx.indent);
    if guard.is_some() {
        append!(*ts, "{}const {emit_props} = (() => {{\n", ctx.indent);
        append!(
            *ts,
            "{call_indent}// @ts-ignore Inference-only guard; authored binding checks own diagnostics.\n"
        );
        append!(
            *ts,
            "{call_indent}if ({}) return (undefined as unknown as {resolver_type})({{\n",
            guard.as_deref().unwrap()
        );
    } else {
        append!(
            *ts,
            "{}const {emit_props} = (undefined as unknown as {resolver_type})({{\n",
            ctx.indent,
        );
    }
    for prop in &usage.props {
        if prop.name_is_dynamic || prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        let Some(generated_value) = generated_prop_value(prop, ctx.template_prop_names) else {
            continue;
        };
        let camel_prop_name = to_camel_case(prop.name.as_str());
        if is_inline_callback_prop(prop) {
            append!(
                *ts,
                "{call_indent}  // @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n"
            );
        }
        append!(
            *ts,
            "{call_indent}  \"{camel_prop_name}\": {},\n",
            generated_value.as_str(),
        );
    }
    append!(*ts, "{call_indent}}});\n");
    if guard.is_some() {
        append!(
            *ts,
            "{call_indent}return undefined as never;\n{}}})();\n",
            ctx.indent
        );
    }
    append!(
        *ts,
        "{}type {inferred_args} = typeof {emit_props} extends {{ {}?: (...args: infer __A) => any }} ? __A : unknown[];\n",
        ctx.indent,
        ctx.prop_key,
    );
    Some(inferred_args)
}

pub(super) fn find_component_usage_for_event<'a>(
    summary: &'a Croquis,
    component_name: &str,
    data: &EventHandlerScopeData,
    scope: &Scope,
) -> Option<(usize, &'a ComponentUsage)> {
    summary
        .component_usages
        .iter()
        .enumerate()
        .find(|(_, usage)| {
            usage.name.as_str() == component_name
                && usage.events.iter().any(|event| {
                    !event.name_is_dynamic
                        && event.name.as_str() == data.event_name.as_str()
                        && event_matches_scope(usage, event.start, event.end, data, scope)
                })
        })
}

fn event_matches_scope(
    usage: &ComponentUsage,
    event_start: u32,
    event_end: u32,
    data: &EventHandlerScopeData,
    scope: &Scope,
) -> bool {
    let exact = event_start == scope.span.start && event_end == scope.span.end;
    let event_contains_scope = event_start <= scope.span.start && scope.span.end <= event_end;
    let scope_in_usage = usage.start <= scope.span.start && scope.span.end <= usage.end;
    let same_handler = usage.events.iter().any(|event| {
        event.start == event_start
            && event.end == event_end
            && data.handler_expression.as_deref() == event.handler.as_deref()
    });
    exact || event_contains_scope || (scope_in_usage && same_handler)
}
