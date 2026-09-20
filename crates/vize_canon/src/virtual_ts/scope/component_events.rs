//! Component event listener type generation.

use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
mod generic_inference;
mod handler_context;
mod inference_helpers;
pub(crate) use inference_helpers::emit_event_inference_helpers;

use vize_carton::{CompactString, FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, EventHandlerScopeData, Scope, naming::to_pascal_case};

use crate::virtual_ts::component_reference::component_binding_reference;
use crate::virtual_ts::helpers::{
    push_ts_string_literal, to_safe_identifier, to_safe_identifier_fragment,
};
use crate::virtual_ts::types::VirtualTsOptions;
use generic_inference::{
    EmitInferenceContext, find_component_usage_for_event, generate_inferred_emit_args,
};
use handler_context::requires_unresolved_handler_implicit_any;

pub(super) struct ComponentEventTypes {
    pub(super) event_type: String,
    pub(super) handler_type: Option<String>,
    pub(super) handler_type_expr: Option<String>,
    pub(super) listener_type: String,
    pub(super) listener_type_expr: String,
}

pub(super) struct ComponentEventTypeContext<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) virtual_ts_options: &'a VirtualTsOptions,
    pub(super) data: &'a EventHandlerScopeData,
    pub(super) scope: &'a Scope,
    pub(super) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    pub(super) template_binding_access: &'a TemplateBindingAccess,
    pub(super) legacy_vue2: bool,
    pub(super) needs_typed_handler_assignment: bool,
    /// `fallthroughAttributes`: a listener the child neither declares nor
    /// emits may still be typed from its root's forwarded props.
    pub(super) fallthrough_listeners: bool,
    pub(super) indent: &'a str,
}

pub(super) fn generate_component_event_types(
    ts: &mut String,
    ctx: ComponentEventTypeContext<'_>,
) -> Option<ComponentEventTypes> {
    let ComponentEventTypeContext {
        summary,
        virtual_ts_options,
        data,
        scope,
        syntactic_type_only_imported_names,
        template_binding_access,
        legacy_vue2,
        needs_typed_handler_assignment,
        fallthrough_listeners,
        indent,
    } = ctx;
    let component_name = data.target_component.as_ref()?;
    let scope_id = scope.id.as_u32();
    let safe_event_name = to_safe_identifier(data.event_name.as_str());
    let component_ref = component_binding_reference(
        summary,
        virtual_ts_options,
        syntactic_type_only_imported_names,
        component_name.as_str(),
    );
    let component_type_name = to_safe_identifier_fragment(component_name.as_str());
    let pascal_event = to_pascal_case(data.event_name.as_str());
    let on_handler = cstr!("on{pascal_event}");
    let prop_key = if on_handler.contains(':') {
        cstr!("\"{}\"", on_handler.as_str())
    } else {
        on_handler
    };
    let prop_args = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_prop_args");
    // Vue's published EmitsToProps retains authored hyphens (onFoo-bar),
    // while generated SFC contracts expose the camelized listener (onFooBar).
    // Only the spelling actually written by the user admits the raw alias.
    let missing_prop_args = if data.event_name.contains('-') {
        let mut raw_handler = String::from("on");
        let mut chars = data.event_name.chars();
        if let Some(first) = chars.next() {
            raw_handler.extend(first.to_uppercase());
        }
        raw_handler.push_str(chars.as_str());
        let mut raw_key = String::default();
        push_ts_string_literal(&mut raw_key, &raw_handler);
        cstr!("__P extends {{ {raw_key}?: (...args: infer __A) => any }} ? __A : unknown[]")
    } else {
        String::from("unknown[]")
    };
    let static_emit_args =
        cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_static_emit_args");
    let emit_args = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_emit_args");
    let fallthrough_args =
        cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_fallthrough_args");
    let args_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_args");
    let return_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_return");
    let event_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_event");
    let listener_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_listener");

    append!(
        *ts,
        "{indent}type {prop_args} = typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }}\n",
    );
    append!(
        *ts,
        "{indent}  ? __P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : {missing_prop_args}\n",
    );
    append!(
        *ts,
        "{indent}  : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any\n",
    );
    append!(
        *ts,
        "{indent}    ? __P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : {missing_prop_args}\n",
    );
    append!(*ts, "{indent}    : unknown[];\n");
    // Under `fallthroughAttributes`, a listener the child neither declares
    // nor emits can still be a fallthrough attribute of its root: `<child
    // @input>` over a `<input />` root receives the `InputEvent` that
    // `NativeElements['input']` declares for `onInput`, and a component root
    // contributes the listener props of the component it renders. The open
    // `Record<string, unknown>` surface resolves to `unknown[]`, so an
    // unresolved root keeps the untyped listener it has today. Without the
    // option the listener stays untyped, as it does under `vue-tsc`.
    let unresolved_args: &str = if fallthrough_listeners {
        append!(
            *ts,
            "{indent}type {fallthrough_args} = typeof {component_ref} extends {{ readonly __vizeFallthroughProps?: infer __F }}\n",
        );
        // A forwarded listener with no parameters is left untyped rather than
        // imposed: the root's own declaration may be looser than the child's
        // documented payload, and a zero-parameter contextual type would make
        // an authored `(event) => ...` callback implicitly `any` (TS7006).
        append!(
            *ts,
            "{indent}  ? __VizeIsAny<__F> extends true ? unknown[] : NonNullable<__F> extends {{ {prop_key}?: (...args: infer __A) => any }} ? (__A extends [] ? unknown[] : __A) : unknown[]\n",
        );
        append!(*ts, "{indent}  : unknown[];\n");
        fallthrough_args.as_str()
    } else {
        "unknown[]"
    };

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
            template_binding_access,
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
            "{indent}type {args_type} = unknown[] extends {inferred} ? (unknown[] extends {prop_args} ? (unknown[] extends {emit_args} ? {unresolved_args} : {emit_args}) : {prop_args}) : {inferred};\n",
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
            "{indent}  : unknown[];\n{indent}type {args_type} = unknown[] extends {prop_args} ? (unknown[] extends {emit_args} ? {unresolved_args} : {emit_args}) : {prop_args};\n",
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
    // The modern listener returns what the child's declared listener prop
    // returns: `any` for the handler props Vue's own `EmitFn`/Volar synthesis
    // produce (`(user: User) => any`), so an emit-payload mismatch elaborates
    // with the same expected type `vue-tsc` prints (#3889), and the authored
    // return type of a prop such as `onClick?: () => number`, so a handler
    // that returns nothing is rejected as it is under `vue-tsc`.
    let listener_type_expr = if legacy_vue2 {
        cstr!("(...args: {listener_args_type}) => unknown")
    } else {
        append!(
            *ts,
            "{indent}type {return_type} = unknown[] extends {prop_args} ? any : typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }}\n",
        );
        append!(
            *ts,
            "{indent}  ? (__P extends {{ {prop_key}?: (...args: any[]) => infer __R }} ? __R : any)\n",
        );
        append!(
            *ts,
            "{indent}  : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any\n",
        );
        append!(
            *ts,
            "{indent}    ? (__P extends {{ {prop_key}?: (...args: any[]) => infer __R }} ? __R : any)\n",
        );
        append!(*ts, "{indent}    : any;\n");
        cstr!(
            "unknown[] extends {args_type} ? ((...args: any[]) => any) : ((...args: {listener_args_type}) => {return_type})"
        )
    };
    let has_script_component_binding = summary.binding_spans.contains_key(component_name.as_str());
    let requires_unresolved_handler =
        requires_unresolved_handler_implicit_any(summary, component_name, data, scope);
    let handler_type_expr = (!legacy_vue2
        && needs_typed_handler_assignment
        && (has_script_component_binding || requires_unresolved_handler))
        .then(|| {
            if has_script_component_binding {
                cstr!("unknown[] extends {args_type} ? ((...args: any[]) => any) : {listener_type}")
            } else {
                cstr!("unknown[] extends {args_type} ? unknown : {listener_type}")
            }
        });
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
