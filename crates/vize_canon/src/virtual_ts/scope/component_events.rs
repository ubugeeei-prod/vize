//! Component event listener type generation.

mod generic_inference;
mod handler_context;
mod reference;

use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::croquis::PassedProp;
use vize_croquis::{
    Croquis, EventHandlerScopeData, Scope, analyzer::strip_js_comments, naming::to_pascal_case,
};

use crate::virtual_ts::{
    expressions::rewrite_reserved_template_prop,
    helpers::{to_safe_identifier, to_safe_identifier_fragment},
};
use generic_inference::{
    EmitInferenceContext, find_component_usage_for_event, generate_inferred_emit_args,
};
use handler_context::requires_unresolved_handler_implicit_any;
use reference::component_reference_expression;

pub(super) struct ComponentEventTypes {
    pub(super) event_type: String,
    pub(super) handler_type: Option<String>,
    pub(super) handler_type_expr: Option<String>,
    pub(super) listener_type: String,
    pub(super) listener_type_expr: String,
}

pub(super) fn generate_component_event_types(
    ts: &mut String,
    summary: &Croquis,
    data: &EventHandlerScopeData,
    scope: &Scope,
    template_prop_names: &FxHashSet<String>,
    legacy_vue2: bool,
    indent: &str,
) -> Option<ComponentEventTypes> {
    let component_name = data.target_component.as_ref()?;
    let scope_id = scope.id.as_u32();
    let safe_event_name = to_safe_identifier(data.event_name.as_str());
    let component_ref = component_reference_expression(component_name.as_str());
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

    // In legacy Vue 2 mode the listener rest args go through the loose emit
    // wrapper so object payload callbacks stay permissive; otherwise they use
    // the resolved emit argument tuple directly.
    let listener_args_type = if legacy_vue2 {
        let legacy_args_type =
            cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_legacy_args");
        append!(
            *ts,
            "{indent}type {event_type} = {args_type} extends [] ? any : unknown[] extends {args_type} ? any : __VizeVue2LooseEventArg<{args_type}[0]>;\n",
        );
        append!(
            *ts,
            "{indent}type {legacy_args_type} = {args_type} extends [] ? any[] : unknown[] extends {args_type} ? any[] : __VizeVue2LooseEmitArgs<{args_type}>;\n",
        );
        legacy_args_type
    } else {
        append!(
            *ts,
            "{indent}type {event_type} = {args_type} extends [] ? any : unknown[] extends {args_type} ? any : {args_type}[0];\n",
        );
        args_type.clone()
    };
    // The modern listener returns `any`, matching the handler props Vue's own
    // `EmitFn`/Volar synthesis produce (`(user: User) => any`); an emit-payload
    // mismatch then elaborates with the same expected type `vue-tsc` prints.
    // `unknown` behaves identically in the expected-return position, so this is
    // display parity only (#3889).
    let listener_type_expr = if legacy_vue2 {
        cstr!("(...args: {listener_args_type}) => unknown")
    } else {
        cstr!(
            "unknown[] extends {args_type} ? ((...args: any[]) => any) : ((...args: {listener_args_type}) => any)"
        )
    };
    let handler_type_expr = (!legacy_vue2
        && requires_unresolved_handler_implicit_any(summary, component_name, data, scope))
    .then(|| cstr!("unknown[] extends {args_type} ? unknown : {listener_type}"));
    let handler_type = handler_type_expr
        .as_ref()
        .map(|_| cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_handler"));
    Some(ComponentEventTypes {
        event_type,
        handler_type,
        handler_type_expr,
        listener_type,
        listener_type_expr,
    })
}

fn push_ts_string_literal(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

pub(super) fn generated_prop_value(
    prop: &PassedProp,
    template_prop_names: &FxHashSet<String>,
) -> Option<String> {
    if !prop.is_dynamic {
        let mut value = String::default();
        if let Some(static_value) = prop.value.as_ref() {
            push_ts_string_literal(&mut value, static_value.as_str());
        } else {
            value.push_str("true");
        }
        return Some(value);
    }

    let value = strip_js_comments(prop.value.as_ref()?.as_str());
    let trimmed_value = value.as_ref().trim();
    let rewritten_value = rewrite_reserved_template_prop(trimmed_value, template_prop_names);
    Some(rewritten_value.as_ref().map_or_else(
        || String::from(value.as_ref()),
        |s| String::from(s.as_str()),
    ))
}
