//! Component event listener type generation.

use vize_carton::{FxHashSet, String, append, cstr, profile};
use vize_croquis::{
    Croquis, EventHandlerScopeData, Scope, ScopeData, analysis::ComponentUsage,
    analyzer::strip_js_comments, naming::to_pascal_case,
};

use crate::virtual_ts::{
    expressions::rewrite_reserved_template_prop,
    helpers::{get_dom_event_type, to_camel_case, to_safe_identifier, to_safe_identifier_fragment},
};

pub(super) struct ComponentEventTypes {
    pub(super) event_type: String,
    pub(super) args_type: String,
    pub(super) listener_type: String,
}

pub(super) fn generate_component_event_types(
    ts: &mut String,
    summary: &Croquis,
    data: &EventHandlerScopeData,
    scope: &Scope,
    template_prop_names: &FxHashSet<String>,
    _template_syntax_quirks: bool,
    indent: &str,
) -> Option<ComponentEventTypes> {
    let component_name = data.target_component.as_ref()?;
    let scope_id = scope.id.as_u32();
    let safe_event_name = to_safe_identifier(data.event_name.as_str());
    let component_ref = to_safe_identifier(component_name.as_str());
    let component_type_name = to_safe_identifier_fragment(component_name.as_str());
    let pascal_event = to_pascal_case(data.event_name.as_str());
    let on_handler = cstr!("on{pascal_event}");
    let prop_key = if on_handler.contains(':') {
        cstr!("\"{}\"", on_handler.as_str())
    } else {
        on_handler
    };
    let prop_args = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_prop_args");
    let static_emit_args =
        cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_static_emit_args");
    let emit_args = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_emit_args");
    let args_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_args");
    let event_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_event");
    let listener_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_listener");

    append!(
        *ts,
        "{indent}type {prop_args} = typeof {component_ref} extends {{ new (): {{ $props: infer __P }} }}\n",
    );
    append!(
        *ts,
        "{indent}  ? __P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[]\n",
    );
    append!(
        *ts,
        "{indent}  : typeof {component_ref} extends (props: infer __P) => any\n",
    );
    append!(
        *ts,
        "{indent}    ? __P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[]\n",
    );
    append!(*ts, "{indent}    : unknown[];\n");

    let inferred_emit_args = generate_inferred_emit_args(
        ts,
        &EmitInferenceContext {
            summary,
            component_name: component_name.as_str(),
            data,
            scope,
            component_ref: &component_ref,
            component_type_name: &component_type_name,
            safe_event_name: &safe_event_name,
            prop_key: &prop_key,
            template_prop_names,
            indent,
        },
    );

    if let Some(ref inferred) = inferred_emit_args {
        append!(
            *ts,
            "{indent}type {static_emit_args} = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }}\n",
        );
        append!(
            *ts,
            "{indent}  ? __EP extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[]\n",
        );
        append!(*ts, "{indent}  : unknown[];\n");
        append!(
            *ts,
            "{indent}type {emit_args} = unknown[] extends {inferred} ? {static_emit_args} : {inferred};\n",
        );
        append!(
            *ts,
            "{indent}type {args_type} = unknown[] extends {inferred} ? (unknown[] extends {prop_args} ? {emit_args} : {prop_args}) : {inferred};\n",
        );
    } else {
        append!(
            *ts,
            "{indent}type {emit_args} = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }}\n",
        );
        append!(
            *ts,
            "{indent}  ? __EP extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[]\n",
        );
        append!(
            *ts,
            "{indent}  : unknown[];\n{indent}type {args_type} = unknown[] extends {prop_args} ? {emit_args} : {prop_args};\n",
        );
    }

    let fallback_event = get_dom_event_type(data.event_name.as_str());
    append!(
        *ts,
        "{indent}type {event_type} = {args_type} extends [] ? any : unknown[] extends {args_type} ? {fallback_event} : {args_type}[0];\n",
    );
    Some(ComponentEventTypes {
        event_type,
        args_type,
        listener_type,
    })
}

struct EmitInferenceContext<'a> {
    summary: &'a Croquis,
    component_name: &'a str,
    data: &'a EventHandlerScopeData,
    scope: &'a Scope,
    component_ref: &'a str,
    component_type_name: &'a str,
    safe_event_name: &'a str,
    prop_key: &'a str,
    template_prop_names: &'a FxHashSet<String>,
    indent: &'a str,
}

fn generate_inferred_emit_args(ts: &mut String, ctx: &EmitInferenceContext<'_>) -> Option<String> {
    if !is_local_vue_component_binding(ctx.summary, ctx.component_name) {
        return None;
    }
    let (usage_idx, usage) =
        find_component_usage_for_event(ctx.summary, ctx.component_name, ctx.data, ctx.scope)?;
    if !usage.props.iter().any(|prop| {
        prop.name.as_str() != "key"
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
        "{}type {resolver_type} = typeof {} extends {{ __vizeResolveEmitProps?: infer __F }} ? (__F extends (...args: any[]) => any ? __F : (props: any) => {{}}) : (props: any) => {{}};\n",
        ctx.indent,
        ctx.component_ref,
    );
    append!(
        *ts,
        "{}const {emit_props} = (undefined as unknown as {resolver_type})({{\n",
        ctx.indent,
    );
    for prop in &usage.props {
        if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        let Some(ref value) = prop.value else {
            continue;
        };
        if !prop.is_dynamic {
            continue;
        }
        let value = profile!(
            "canon.virtual_ts.emit_payload.strip_comments",
            strip_js_comments(value.as_str())
        );
        let trimmed_value = value.as_ref().trim();
        let rewritten_value =
            rewrite_reserved_template_prop(trimmed_value, ctx.template_prop_names);
        let generated_value = rewritten_value
            .as_ref()
            .map_or_else(|| value.as_ref(), |s| s.as_str());
        let camel_prop_name = to_camel_case(prop.name.as_str());
        append!(
            *ts,
            "{}  \"{camel_prop_name}\": {generated_value},\n",
            ctx.indent
        );
    }
    append!(*ts, "{}}});\n", ctx.indent);
    append!(
        *ts,
        "{}type {inferred_args} = typeof {emit_props} extends {{ {}?: (...args: infer __A) => any }} ? __A : unknown[];\n",
        ctx.indent,
        ctx.prop_key,
    );
    Some(inferred_args)
}

fn is_local_vue_component_binding(summary: &Croquis, component_name: &str) -> bool {
    let Some((binding_start, binding_end)) = summary.binding_spans.get(component_name) else {
        return false;
    };

    summary.scopes.iter().any(|scope| {
        scope.span.start <= *binding_start
            && *binding_end <= scope.span.end
            && matches!(
                scope.data(),
                ScopeData::ExternalModule(data)
                    if !data.is_type_only && data.source.as_str().ends_with(".vue")
            )
    })
}

fn find_component_usage_for_event<'a>(
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
                    event.name.as_str() == data.event_name.as_str()
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
