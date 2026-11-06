//! Component event listener type generation.

use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::{expressions::ComponentPropSource, types::VizeMapping};
mod emit_props_gate;
mod emit_resolver_share;
mod generic_inference;
mod handler_context;
mod inference_helpers;
mod listener_relations;
mod listener_return;
mod native_fallthrough;
mod prop_array_share;
pub(crate) use emit_props_gate::{emit_gate_aliases, gates_inference};
pub(crate) use emit_resolver_share::{
    flush as flush_emit_resolvers, reset as reset_emit_resolvers,
};
pub(crate) use inference_helpers::emit_event_inference_helpers;

use vize_carton::{CompactString, FxHashMap, FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, EventHandlerScopeData, Scope, naming::to_pascal_case};

use super::context::{ComponentContractKey, EventContract, EventContractKey};
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
    /// A loose resolver has no contextual parameter signature, unlike a declared zero-argument emit.
    pub(super) contextless_listener: Option<String>,
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
    pub(super) share_listener_relations: bool,
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
    /// A props-independent emit-props call checks its literal against `any`.
    pub(super) gated_inference: bool,
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
        share_listener_relations,
        needs_typed_handler_assignment,
        fallthrough_listeners,
        native_fallthrough_only,
        explicit_generic,
        enclosing_vif_guard,
        event_inference_usages,
        event_contracts,
        indent,
        gated_inference,
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
    let mut listener_key = String::default();
    push_ts_string_literal(&mut listener_key, on_handler.as_str());
    let prop_key = if on_handler.contains(':') {
        cstr!("\"{}\"", on_handler.as_str())
    } else {
        on_handler
    };
    let component_contract = ComponentContractKey {
        component_type_name: component_type_name.clone(),
        component_ref: component_ref.clone(),
        legacy_vue2,
        fallthrough_listeners,
        native_fallthrough_only,
        enclosing_vif_guard: enclosing_vif_guard.map(String::from),
        parent_scope: scope.parent().map(|parent| parent.as_u32()),
        // Aliases of the first usage are declared inside its guarded block;
        // a later block with the same guard cannot see them.
        chained_guard_block: super::open_chained_vif_guard_block(),
    };
    let claim = |event: Option<EventContract>| {
        let key = EventContractKey {
            component: component_contract.clone(),
            event,
        };
        let mut contracts = event_contracts.borrow_mut();
        match contracts.get(&key) {
            Some(&contract_scope_id) => (contract_scope_id, false),
            None => {
                contracts.insert(key, scope_id);
                (scope_id, true)
            }
        }
    };
    let (component_scope_id, first_component) = claim(None);
    let (contract_scope_id, first_contract) = claim(Some(EventContract {
        name: String::from(data.event_name.as_str()),
        typed_handler: needs_typed_handler_assignment && has_script_component_binding,
    }));
    // Every event of this component reads the same props, emit props and
    // fallthrough root: derive each once and unwrap the one-tuple per event.
    let component_aliases = cstr!("__{component_type_name}_{component_scope_id}_");
    let props_box = cstr!("{component_aliases}_props");
    let emit_props_box = cstr!("{component_aliases}_emit_props");
    let fallthrough_box = cstr!("{component_aliases}_fallthrough");
    let library = cstr!("{component_aliases}_library");
    let relations = listener_relations::ListenerRelations::new(
        &component_aliases,
        hoist_shared_preamble && share_listener_relations,
    );
    let prop_args =
        cstr!("__{component_type_name}_{contract_scope_id}_{safe_event_name}_prop_args");
    // Vue's published EmitsToProps retains authored hyphens (onFoo-bar),
    // while generated SFC contracts expose the camelized listener (onFooBar).
    // Only the spelling actually written by the user admits the raw alias.
    let raw_listener_key = if data.event_name.contains('-') {
        let mut raw_handler = String::from("on");
        let mut chars = data.event_name.chars();
        if let Some(first) = chars.next() {
            raw_handler.extend(first.to_uppercase());
        }
        raw_handler.push_str(chars.as_str());
        let mut raw_key = String::default();
        push_ts_string_literal(&mut raw_key, &raw_handler);
        raw_key
    } else {
        String::from("never")
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
    if first_component {
        relations.emit(ts, indent);
        // A vue-tsc generic library export instantiates its generics with their
        // constraints here, so its payload comes from the bound-props call below.
        append!(
            *ts,
            "{indent}type {library} = typeof {component_ref} extends {{ __vizeCheck: any }} | {{ readonly __vizeComponentMarker: true }} ? false : typeof {component_ref} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;\n",
        );
        append!(
            *ts,
            "{indent}type {props_box} = typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }} ? [__P] : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any ? [__P] : [];\n",
        );
        append!(
            *ts,
            "{indent}type {emit_props_box} = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }} ? [__EP] : [];\n",
        );
        if native_fallthrough_only {
            native_fallthrough::emit_box(
                ts,
                indent,
                fallthrough_box.as_str(),
                component_ref.as_str(),
                is_any,
            );
        } else if fallthrough_listeners {
            // An `any` root has no typed contract, the same as no root.
            append!(
                *ts,
                "{indent}type {fallthrough_box} = typeof {component_ref} extends {{ readonly __vizeFallthroughProps?: infer __F }} ? {is_any}<__F> extends true ? [] : [__F] : [];\n",
            );
        }
    }
    // Constructor `$props` contains ambient DOM listeners. For a generated,
    // non-generic child, an unowned listener can instead belong to its typed
    // root. Select that contract once for arguments, returns and arrays.
    let contextual_props =
        (fallthrough_listeners && !native_fallthrough_only && !legacy_vue2).then(|| {
        let public_props = cstr!("{component_aliases}_public_props");
        let resolved_props = cstr!("{prop_args}_resolved_props");
        if first_component {
            append!(
                *ts,
                "{indent}type {public_props} = {props_box} extends infer __B ? __B extends [infer __P] ? __P : {{}} : never;\n",
            );
        }
        if first_contract {
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
        "{}<__P, {listener_key}, {raw_listener_key}>",
        relations.args
    );
    if first_contract {
        if let Some(props) = contextual_props.as_ref() {
            append!(
                *ts,
                "{indent}type {prop_args} = {library} extends true ? unknown[] : {props} extends infer __P ? {prop_listener_args} : unknown[];\n",
            );
        } else {
            append!(
                *ts,
                "{indent}type {prop_args} = {library} extends true ? unknown[] : {props_box} extends infer __B ? __B extends [infer __P] ? {prop_listener_args} : unknown[] : never;\n",
            );
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
                fallthrough_box.as_str(),
                prop_listener_args.as_str(),
            );
        }
        fallthrough_args.as_str()
    } else if fallthrough_listeners {
        if first_contract {
            if legacy_vue2 {
                append!(
                    *ts,
                    "{indent}type {fallthrough_args} = {fallthrough_box} extends infer __B ? __B extends [infer __F] ? NonNullable<__F> extends {{ {prop_key}?: (...args: infer __A) => any }} ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[] : never;\n",
                );
            } else {
                append!(
                    *ts,
                    "{indent}type {fallthrough_args} = {fallthrough_box} extends infer __B ? __B extends [infer __F] ? NonNullable<__F> extends infer __P ? ({prop_listener_args}) extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[] : unknown[] : never;\n",
                );
            }
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
            library: &library,
            template_binding_access,
            template_ast,
            source_context,
            enclosing_vif_guard,
            event_inference_usages,
            indent,
            hoist_shared_preamble,
            gated_inference: gated_inference && explicit_generic.is_none(),
            contextual_listener: !legacy_vue2,
        },
    );

    if first_contract {
        append!(
            *ts,
            "{indent}type {static_emit_args} = {emit_props_box} extends infer __B ? __B extends [infer __EP] ? __EP extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[] : unknown[] : never;\n",
        );
    }
    if let Some(inference) = inferred_emit_args.as_ref() {
        let inferred = &inference.arguments;
        // Missing contextual parameters do not mean that runtime emits have
        // zero arguments. Keep the loose sentinel wide, while a declared []
        // keeps its actual empty tuple and its known callback signature.
        let loose_runtime_args = inference
            .contextless_listener
            .as_ref()
            .map(|loose| cstr!("{loose} extends true ? unknown[] : "))
            .unwrap_or_default();
        append!(
            *ts,
            "{indent}type {emit_args} = unknown[] extends {inferred} ? {static_emit_args} : {inferred};\n",
        );
        append!(
            *ts,
            "{indent}type {args_type} = {loose_runtime_args}unknown[] extends {inferred} ? (unknown[] extends {prop_args} ? (unknown[] extends {emit_args} ? {unresolved_args} : {emit_args}) : {prop_args}) : {inferred};\n",
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
            props_box: props_box.as_str(),
            fallthrough_box: fallthrough_box.as_str(),
            prop_key: prop_key.as_str(),
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
                if first_contract {
                    let props_members = cstr!("{component_aliases}_props_members");
                    prop_array_share::emit(
                        ts,
                        prop_array_share::Emit {
                            contracts: event_contracts,
                            component: &component_contract,
                            scope_id,
                            indent,
                            is_any,
                            prop_key: prop_key.as_str(),
                            arrays_type: arrays_type.as_str(),
                            direct_props: contextual_props.as_deref(),
                            props_box: props_box.as_str(),
                            props_members: props_members.as_str(),
                        },
                    );
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
    let contextless_listener =
        inferred_emit_args.and_then(|inference| inference.contextless_listener);
    Some(ComponentEventTypes {
        event_type,
        handler_type,
        handler_annotation,
        contextless_listener,
        handler_type_expr,
        listener_type,
        listener_type_expr,
    })
}
