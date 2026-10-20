use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::{String, append, cstr};
use vize_croquis::{Croquis, EventHandlerScopeData, Scope, analysis::ComponentUsage};

use crate::virtual_ts::{
    expressions::rewrite_reserved_template_binding, helpers::to_camel_case,
    scope::is_inline_callback_prop,
};

use crate::virtual_ts::expressions::{
    ComponentPropSource, append_props_literal, generated_prop_span, map_rewritten_template_binding,
    prop_entry_sub_spans, prop_value_source_range,
};
use crate::virtual_ts::types::VizeMapping;

pub(super) struct EmitInferenceContext<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) component_name: &'a str,
    pub(super) data: &'a EventHandlerScopeData,
    pub(super) scope: &'a Scope,
    pub(super) component_ref: &'a str,
    pub(super) component_type_name: &'a str,
    pub(super) safe_event_name: &'a str,
    pub(super) prop_key: &'a str,
    pub(super) template_binding_access: &'a TemplateBindingAccess,
    pub(super) source_context: ComponentPropSource<'a>,
    pub(super) indent: &'a str,
}

pub(super) fn generate_inferred_emit_args(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &EmitInferenceContext<'_>,
) -> Option<String> {
    let (usage_idx, usage) =
        find_component_usage_for_event(ctx.summary, ctx.component_name, ctx.data, ctx.scope)?;
    if usage.spread_props.is_empty()
        && !usage.props.iter().any(|prop| {
            !prop.name_is_dynamic
                && prop.name.as_str() != "key"
                && prop.name.as_str() != "ref"
                && prop.value.is_some()
        })
    {
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
        "{}type {resolver_type} = __VizeEmitPropsFactory<typeof {}>;\n",
        ctx.indent,
        ctx.component_ref,
    );
    let guard = usage.vif_guard.as_ref().map(|guard| {
        rewrite_reserved_template_binding(guard.as_str(), ctx.template_binding_access)
            .unwrap_or_else(|| guard.clone())
    });
    let guarded_call_indent = guard.as_ref().map(|_| cstr!("{}  ", ctx.indent));
    let call_indent = guarded_call_indent.as_deref().unwrap_or(ctx.indent);
    if let Some(guard) = guard.as_deref() {
        let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
        let guard = isolated.as_str();
        append!(*ts, "{}const {emit_props} = (() => {{\n", ctx.indent);
        append!(
            *ts,
            "{call_indent}// @ts-ignore Inference-only guard; authored binding checks own diagnostics.\n"
        );
        append!(
            *ts,
            "{call_indent}if ({guard}) return (undefined as unknown as {resolver_type})({})(",
            ctx.component_ref,
        );
    } else {
        append!(
            *ts,
            "{}const {emit_props} = (undefined as unknown as {resolver_type})({})(",
            ctx.indent,
            ctx.component_ref,
        );
    }
    if !usage.spread_props.is_empty() {
        let literal_range = append_props_literal(
            ts,
            mappings,
            &usage,
            ctx.template_binding_access,
            ctx.source_context,
            call_indent,
        );
        ts.push_str(");\n");
        // Argument assignability errors cover the whole literal, rather than
        // one spread expression. Preserve the same authored tag owner as the
        // canonical props call; these inference rows remain diagnostics-only.
        if let Some(src_range) = crate::virtual_ts::component_tag::source_range(
            ctx.source_context.template,
            ctx.source_context.offset,
            &usage,
        ) {
            mappings.push(VizeMapping {
                gen_range: literal_range,
                src_range,
                sub_spans: Vec::new(),
            });
        }
    } else {
        ts.push_str("{\n");
        for prop in &usage.props {
            if prop.name_is_dynamic || prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
                continue;
            }
            let Some(generated_value) = generated_prop_span(prop, ctx.template_binding_access)
            else {
                continue;
            };
            let camel_prop_name = to_camel_case(prop.name.as_str());
            if is_inline_callback_prop(prop) {
                append!(
                    *ts,
                    "{call_indent}  // @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n"
                );
            }
            append!(*ts, "{call_indent}  ");
            let entry_start = ts.len();
            append!(*ts, "\"{camel_prop_name}\"");
            let key_end = ts.len();
            ts.push_str(": ");
            let value_start = ts.len();
            ts.push_str(generated_value.text.as_str());
            let entry_end = if generated_value.omits_tail() {
                generated_value.mapped_limit(value_start)
            } else {
                ts.len()
            };
            ts.push_str(",\n");
            mappings.push(VizeMapping {
                gen_range: entry_start..entry_end,
                src_range: (ctx.source_context.offset + prop.start) as usize
                    ..(ctx.source_context.offset + prop.end) as usize,
                sub_spans: prop_entry_sub_spans(
                    ctx.source_context,
                    prop,
                    entry_start..key_end,
                    value_start..entry_end,
                ),
            });
            if !ctx.template_binding_access.is_empty()
                && prop.is_dynamic
                && !is_inline_callback_prop(prop)
                && let Some(value) = prop.value.as_ref()
                && let Some(source) = prop_value_source_range(ctx.source_context, prop)
            {
                map_rewritten_template_binding(
                    ts,
                    mappings,
                    value_start,
                    source.start,
                    value.as_str(),
                    ctx.template_binding_access,
                );
            }
        }
        append!(*ts, "{call_indent}}});\n");
    }
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

pub(super) fn find_component_usage_for_event(
    summary: &Croquis,
    component_name: &str,
    data: &EventHandlerScopeData,
    scope: &Scope,
) -> Option<(usize, ComponentUsage)> {
    vize_croquis::facts::component_usage_list(summary)
        .into_iter()
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
