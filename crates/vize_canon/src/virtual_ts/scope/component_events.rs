//! Component event listener type generation.

use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::{expressions::ComponentPropSource, types::VizeMapping};
mod generic_inference;
mod handler_context;
mod inference_helpers;
mod listener_return;
mod native_fallthrough;
pub(crate) use inference_helpers::emit_event_inference_helpers;

use vize_carton::{CompactString, FxHashMap, FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, EventHandlerScopeData, Scope, naming::to_pascal_case};

use super::context::EventContractKey;
use crate::virtual_ts::component_reference::component_binding_reference_with_resolution;
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
    /// The handler annotation: the alias, plus any array form the child's
    /// listener prop admits, kept outside the alias so TypeScript prints it.
    pub(super) handler_annotation: Option<String>,
    pub(super) listener_type: String,
    pub(super) listener_type_expr: String,
}

pub(super) struct ComponentEventTypeContext<'a, 'template> {
    pub(super) summary: &'a Croquis,
    pub(super) virtual_ts_options: &'a VirtualTsOptions,
    pub(super) data: &'a EventHandlerScopeData,
    pub(super) scope: &'a Scope,
    pub(super) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    pub(super) template_binding_access: &'a TemplateBindingAccess<'a>,
    pub(super) template_ast: Option<&'a vize_relief::RootNode<'template>>,
    pub(super) source_context: ComponentPropSource<'a>,
    pub(super) legacy_vue2: bool,
    pub(super) hoist_shared_preamble: bool,
    pub(super) needs_typed_handler_assignment: bool,
    /// `fallthroughAttributes`: a listener the child neither declares nor
    /// emits may still be typed from its root's forwarded props.
    pub(super) fallthrough_listeners: bool,
    /// An omitted Vue 3 option accepts only an immediate native-root payload.
    pub(super) native_fallthrough_only: bool,
    /// The `@vue-generic` binding this listener's component resolves through.
    pub(super) explicit_generic: Option<&'a str>,
    pub(super) enclosing_vif_guard: Option<&'a str>,
    pub(super) event_inference_usages: &'a std::cell::RefCell<FxHashSet<usize>>,
    pub(super) event_contracts: &'a std::cell::RefCell<FxHashMap<EventContractKey, u32>>,
    pub(super) indent: &'a str,
}

pub(super) fn generate_component_event_types(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: ComponentEventTypeContext<'_, '_>,
) -> Option<ComponentEventTypes> {
    let ComponentEventTypeContext {
        summary,
        virtual_ts_options,
        data,
        scope,
        syntactic_type_only_imported_names,
        template_binding_access,
        template_ast,
        source_context,
        legacy_vue2,
        hoist_shared_preamble,
        needs_typed_handler_assignment,
        fallthrough_listeners,
        native_fallthrough_only,
        explicit_generic,
        enclosing_vif_guard,
        event_inference_usages,
        event_contracts,
        indent,
    } = ctx;
    let component_name = data.target_component.as_ref()?;
    let scope_id = scope.id.as_u32();
    let safe_event_name = to_safe_identifier(data.event_name.as_str());
    let (resolved_component_ref, has_resolved_component_binding) =
        component_binding_reference_with_resolution(
            summary,
            virtual_ts_options,
            syntactic_type_only_imported_names,
            component_name.as_str(),
        );
    let component_ref = explicit_generic.map_or(resolved_component_ref, String::from);
    let component_type_name = to_safe_identifier_fragment(component_name.as_str());
    let is_any = crate::virtual_ts::helpers::shared_types::IS_ANY.select(hoist_shared_preamble);
    let emit_owned_keys =
        crate::virtual_ts::helpers::shared_types::EMIT_OWNED_KEYS.select(hoist_shared_preamble);
    let has_script_component_binding =
        super::super::script_facts::binding_span(summary, component_name.as_str()).is_some();
    let requested_native_fallthrough = native_fallthrough_only;
    let native_fallthrough_only = requested_native_fallthrough && has_resolved_component_binding;
    let fallthrough_listeners =
        fallthrough_listeners && (!requested_native_fallthrough || native_fallthrough_only);
    let pascal_event = to_pascal_case(data.event_name.as_str());
    let on_handler = cstr!("on{pascal_event}");
    let mut owned_listener_key = String::default();
    if fallthrough_listeners && !legacy_vue2 {
        push_ts_string_literal(&mut owned_listener_key, on_handler.as_str());
    }
    let prop_key = if on_handler.contains(':') {
        cstr!("\"{}\"", on_handler.as_str())
    } else {
        on_handler
    };
    let contract_key = EventContractKey {
        component_type_name: component_type_name.clone(),
        component_ref: component_ref.clone(),
        event_name: String::from(data.event_name.as_str()),
        legacy_vue2,
        fallthrough_listeners,
        native_fallthrough_only,
        needs_typed_handler_assignment: needs_typed_handler_assignment
            && has_script_component_binding,
        enclosing_vif_guard: enclosing_vif_guard.map(String::from),
        parent_scope: scope.parent().map(|parent| parent.as_u32()),
        // Aliases of the first usage are declared inside its guarded block;
        // a later block with the same guard cannot see them.
        chained_guard_block: super::open_chained_vif_guard_block(),
    };
    let (contract_scope_id, first_contract) = {
        let mut contracts = event_contracts.borrow_mut();
        if let Some(&contract_scope_id) = contracts.get(&contract_key) {
            (contract_scope_id, false)
        } else {
            contracts.insert(contract_key, scope_id);
            (scope_id, true)
        }
    };
    let prop_args =
        cstr!("__{component_type_name}_{contract_scope_id}_{safe_event_name}_prop_args");
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
        cstr!("__{component_type_name}_{contract_scope_id}_{safe_event_name}_static_emit_args");
    let emit_args = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_emit_args");
    let fallthrough_args =
        cstr!("__{component_type_name}_{contract_scope_id}_{safe_event_name}_fallthrough_args");
    let args_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_args");
    let return_type = cstr!("__{component_type_name}_{contract_scope_id}_{safe_event_name}_return");
    let event_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_event");
    let listener_type = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_listener");
    // A vue-tsc generic library export instantiates its generics with their
    // constraints here; its listener stays untyped rather than wrong.
    if first_contract {
        append!(
            *ts,
            "{indent}type {prop_args}_library = typeof {component_ref} extends {{ __vizeCheck: any }} | {{ readonly __vizeComponentMarker: true }} ? false : typeof {component_ref} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;\n",
        );
    }
    // Constructor `$props` contains ambient DOM listeners. For a generated,
    // non-generic child, an unowned listener can instead belong to its typed
    // root. Select that contract once for arguments, returns and arrays.
    let contextual_props =
        (fallthrough_listeners && !native_fallthrough_only && !legacy_vue2).then(|| {
        let public_props = cstr!("{prop_args}_public_props");
        let resolved_props = cstr!("{prop_args}_resolved_props");
        if first_contract {
            append!(
                *ts,
                "{indent}type {public_props} = typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }} ? __P : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any ? __P : {{}};\n",
            );
            append!(
                *ts,
                "{indent}type {resolved_props} = typeof {component_ref} extends {{ readonly __vizeComponentMarker: true }} ? typeof {component_ref} extends {{ __vizeCheck: unknown }} ? {public_props} : {owned_listener_key} extends {emit_owned_keys}<typeof {component_ref}> ? {public_props} : unknown[] extends {fallthrough_args} ? {public_props} : typeof {component_ref} extends {{ readonly __vizeFallthroughProps?: infer __F }} ? NonNullable<__F> : {public_props} : {public_props};\n",
            );
        }
        resolved_props
    });
    // A listener prop may also accept an array of listeners (`onClick?: Fn |
    // Fn[]`, as Nuxt UI's Button declares); its function member still
    // supplies the payload.
    let prop_listener_args = cstr!(
        "__P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : __P extends {{ {prop_key}?: infer __H }} ? [Extract<NonNullable<__H>, (...args: any) => any>] extends [never] ? {missing_prop_args} : Extract<NonNullable<__H>, (...args: any) => any> extends (...args: infer __A) => any ? __A : {missing_prop_args} : {missing_prop_args}"
    );
    if first_contract {
        if let Some(props) = contextual_props.as_ref() {
            append!(
                *ts,
                "{indent}type {prop_args} = {prop_args}_library extends true ? unknown[] : {props} extends infer __P ? {prop_listener_args} : unknown[];\n",
            );
        } else {
            append!(
                *ts,
                "{indent}type {prop_args} = {prop_args}_library extends true ? unknown[] : typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }}\n",
            );
            append!(*ts, "{indent}  ? {prop_listener_args}\n");
            append!(
                *ts,
                "{indent}  : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any\n",
            );
            append!(*ts, "{indent}    ? {prop_listener_args}\n");
            append!(*ts, "{indent}    : unknown[];\n");
        }
    }
    // Explicit `fallthroughAttributes` admits a forwarded listener contract.
    // When Vue 3 omits the option, only an immediate native-root marker admits
    // that payload. Explicit false, Vue 2, component roots and open no-root or
    // unresolved surfaces retain their untyped listener contract.
    let unresolved_args: &str = if native_fallthrough_only {
        if first_contract {
            // A forwarded listener with no parameters is left untyped rather than
            // imposed: the root's own declaration may be looser than the child's
            // documented payload, and a zero-parameter contextual type would make
            // an authored `(event) => ...` callback implicitly `any` (TS7006).
            native_fallthrough::emit_args(
                ts,
                indent,
                fallthrough_args.as_str(),
                component_ref.as_str(),
                prop_listener_args.as_str(),
                is_any,
            );
        }
        fallthrough_args.as_str()
    } else if fallthrough_listeners {
        if first_contract {
            append!(
                *ts,
                "{indent}type {fallthrough_args} = typeof {component_ref} extends {{ readonly __vizeFallthroughProps?: infer __F }}\n",
            );
            if legacy_vue2 {
                append!(
                    *ts,
                    "{indent}  ? {is_any}<__F> extends true ? unknown[] : NonNullable<__F> extends {{ {prop_key}?: (...args: infer __A) => any }} ? (__A extends [] ? unknown[] : __A) : unknown[]\n",
                );
            } else {
                append!(
                    *ts,
                    "{indent}  ? {is_any}<__F> extends true ? unknown[] : NonNullable<__F> extends infer __P ? ({prop_listener_args}) extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[]\n",
                );
            }
            append!(*ts, "{indent}  : unknown[];\n");
        }
        fallthrough_args.as_str()
    } else {
        "unknown[]"
    };

    let inferred_emit_args = generate_inferred_emit_args(
        ts,
        mappings,
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
            template_ast,
            source_context,
            enclosing_vif_guard,
            event_inference_usages,
            indent,
            hoist_shared_preamble,
        },
    );

    if first_contract {
        append!(
            *ts,
            "{indent}type {static_emit_args} = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }}\n",
        );
        append!(
            *ts,
            "{indent}  ? __EP extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[]\n",
        );
        append!(*ts, "{indent}  : unknown[];\n");
    }
    if let Some(ref inferred) = inferred_emit_args {
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
            "{indent}type {emit_args} = {static_emit_args};\n{indent}type {args_type} = unknown[] extends {prop_args} ? (unknown[] extends {emit_args} ? {unresolved_args} : {emit_args}) : {prop_args};\n",
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
            "{indent}type {event_type} = {args_type} extends infer __A extends unknown[] ? [__A] extends [[]] ? any : unknown[] extends __A ? any : $Vue2LooseEventArg<__A[0]> : never;\n",
        );
        append!(
            *ts,
            "{indent}type {legacy_args_type} = {args_type} extends [] ? any[] : unknown[] extends {args_type} ? any[] : $Vue2LooseEmitArgs<{args_type}>;\n",
        );
        legacy_args_type
    } else {
        append!(
            *ts,
            "{indent}type {event_type} = {args_type} extends infer __A extends unknown[] ? [__A] extends [[]] ? any : unknown[] extends __A ? any : __A[0] : never;\n",
        );
        args_type.clone()
    };
    // The modern listener returns what the child's declared listener prop
    // returns: `any` for the handler props Vue's own `EmitFn`/Volar synthesis
    // produce (`(user: User) => any`), so an emit-payload mismatch elaborates
    // with the same expected type `vue-tsc` prints (#3889), and the authored
    // return type of a prop such as `onClick?: () => number`, so a handler
    // that returns nothing is rejected as it is under `vue-tsc`.
    let listener_type_expr = listener_return::emit(
        ts,
        listener_return::Context {
            legacy_vue2,
            first_contract,
            native_fallthrough_only,
            contextual_props: contextual_props.as_deref(),
            indent,
            listener_args_type: listener_args_type.as_str(),
            args_type: args_type.as_str(),
            return_type: return_type.as_str(),
            prop_args: prop_args.as_str(),
            component_ref: component_ref.as_str(),
            prop_key: prop_key.as_str(),
            is_any,
        },
    );
    let requires_unresolved_handler =
        requires_unresolved_handler_implicit_any(summary, component_name, data, scope);
    let mut prop_arrays = None;
    let handler_type_expr = (!legacy_vue2
        && needs_typed_handler_assignment
        && (has_script_component_binding || requires_unresolved_handler))
        .then(|| {
            if has_script_component_binding {
                // Vue invokes every listener of an array; accept one only
                // where the child's own listener prop admits it, as vue-tsc.
                let arrays_type = cstr!(
                    "__{component_type_name}_{contract_scope_id}_{safe_event_name}_prop_arrays"
                );
                let arrays = cstr!(
                    "__P extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, readonly unknown[]> : never"
                );
                if first_contract {
                    if let Some(props) = contextual_props.as_ref() {
                        append!(
                            *ts,
                            "{indent}type {arrays_type} = {props} extends infer __P ? {arrays} : never;\n",
                        );
                    } else {
                        append!(
                            *ts,
                            "{indent}type {arrays_type} = typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }} ? {arrays} : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any ? {arrays} : never;\n",
                        );
                    }
                }
                prop_arrays = Some(arrays_type);
                cstr!("unknown[] extends {args_type} ? ((...args: any[]) => any) : {listener_type}")
            } else {
                cstr!("unknown[] extends {args_type} ? unknown : {listener_type}")
            }
        });
    let handler_type = handler_type_expr
        .as_ref()
        .map(|_| cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_handler"));
    let handler_annotation = handler_type.as_ref().map(|handler| match &prop_arrays {
        Some(arrays) => cstr!("{handler} | {arrays}"),
        None => handler.clone(),
    });
    Some(ComponentEventTypes {
        event_type,
        handler_type,
        handler_annotation,
        handler_type_expr,
        listener_type,
        listener_type_expr,
    })
}
