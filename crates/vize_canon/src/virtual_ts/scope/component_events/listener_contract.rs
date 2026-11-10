//! One shared listener contract per component event.
//!
//! The per-event alias chain (`_prop_args`, `_fallthrough_args`,
//! `_static_emit_args`, `_args`, `_event`, `_return`, `_prop_arrays`, …) is the
//! body of `$ListenerContract`, a shared SDK helper. Each use declares one
//! alias that passes the component's boxes, the event key and its inferred
//! payload, so TypeScript instantiates the contract once per argument list
//! across the program instead of once per use. The component box aliases and
//! the inference-only call stay per use. Each box names a non-distributive
//! shared `$Usage*` helper, so it reads `typeof Component` exactly as the chain
//! did, including its `any`, `never` and union cases.

use vize_carton::{FxHashSet, String, cstr};
use vize_croquis::{Croquis, EventHandlerScopeData, Scope, naming::to_pascal_case};

use super::ComponentEventTypes;
use super::generic_inference::{EmitInferenceContext, generate_inferred_emit_args};
use super::handler_context::requires_unresolved_handler_implicit_any;
use crate::virtual_ts::expressions::ComponentPropSource;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::types::VizeMapping;

/// A listener prop's keys: the TS string literal, the object-type key, and the
/// raw kebab spelling or `never`.
pub(super) struct Keys {
    pub(super) listener_key: String,
    pub(super) prop_key: String,
    pub(super) raw_listener_key: String,
}

pub(super) fn keys(event_name: &str) -> Keys {
    let on_handler = cstr!("on{}", to_pascal_case(event_name));
    let mut listener_key = String::default();
    crate::virtual_ts::helpers::push_ts_string_literal(&mut listener_key, on_handler.as_str());
    let prop_key = if on_handler.contains(':') {
        cstr!("\"{}\"", on_handler.as_str())
    } else {
        on_handler
    };
    // Vue's published EmitsToProps retains authored hyphens (onFoo-bar),
    // while generated SFC contracts expose the camelized listener (onFooBar).
    // Only the spelling actually written by the user admits the raw alias.
    let raw_listener_key = if event_name.contains('-') {
        let mut raw_handler = String::from("on");
        let mut chars = event_name.chars();
        if let Some(first) = chars.next() {
            raw_handler.extend(first.to_uppercase());
        }
        raw_handler.push_str(chars.as_str());
        let mut raw_key = String::default();
        crate::virtual_ts::helpers::push_ts_string_literal(&mut raw_key, &raw_handler);
        raw_key
    } else {
        String::from("never")
    };
    Keys {
        listener_key,
        prop_key,
        raw_listener_key,
    }
}

/// The contract covers Vue 3 listeners of a closed template whose shared SDK
/// is hoisted and whose `Extract`/`NonNullable` are the lib's (the same
/// condition the shared `$ListenerArgs` relation needs). Explicit
/// `fallthroughAttributes` (the contextual root props), Vue 2 and
/// `@vue-generic` bindings keep the alias chain.
pub(super) const fn applies(
    legacy_vue2: bool,
    shared_relations: bool,
    gated_inference: bool,
    explicit_generic: bool,
    contextual_props: bool,
) -> bool {
    !legacy_vue2 && shared_relations && gated_inference && !explicit_generic && !contextual_props
}

pub(super) struct Contract<'a, 'template> {
    pub(super) summary: &'a Croquis,
    pub(super) component_name: &'a str,
    pub(super) data: &'a EventHandlerScopeData,
    pub(super) scope: &'a Scope,
    pub(super) component_ref: &'a str,
    pub(super) component_type_name: &'a str,
    pub(super) safe_event_name: &'a str,
    pub(super) prop_key: &'a str,
    pub(super) library: &'a str,
    pub(super) props_box: &'a str,
    pub(super) emit_props_box: &'a str,
    /// The native root's box, or `None` when no fallthrough payload applies.
    pub(super) fallthrough_box: Option<&'a str>,
    pub(super) listener_key: &'a str,
    pub(super) raw_listener_key: &'a str,
    pub(super) template_binding_access: &'a TemplateBindingAccess<'a>,
    pub(super) template_ast: Option<&'a vize_relief::RootNode<'template>>,
    pub(super) source_context: ComponentPropSource<'a>,
    pub(super) inference_outer_vif_guard: Option<&'a str>,
    pub(super) event_inference_usages: &'a std::cell::RefCell<FxHashSet<usize>>,
    pub(super) indent: &'a str,
    pub(super) needs_typed_handler_assignment: bool,
    pub(super) has_script_component_binding: bool,
    pub(super) scope_id: u32,
}

pub(super) fn emit(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    contract: Contract<'_, '_>,
) -> ComponentEventTypes {
    let inferred = generate_inferred_emit_args(
        ts,
        mappings,
        &EmitInferenceContext {
            summary: contract.summary,
            component_name: contract.component_name,
            data: contract.data,
            scope: contract.scope,
            component_ref: contract.component_ref,
            component_type_name: contract.component_type_name,
            safe_event_name: contract.safe_event_name,
            prop_key: contract.prop_key,
            library: contract.library,
            template_binding_access: contract.template_binding_access,
            template_ast: contract.template_ast,
            source_context: contract.source_context,
            enclosing_vif_guard: contract.inference_outer_vif_guard,
            event_inference_usages: contract.event_inference_usages,
            indent: contract.indent,
            hoist_shared_preamble: true,
            gated_inference: true,
            contextual_listener: true,
        },
    );
    let Contract {
        component_type_name,
        safe_event_name,
        scope_id,
        indent,
        library,
        props_box,
        emit_props_box,
        listener_key,
        raw_listener_key,
        ..
    } = contract;
    let alias = cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_contract");
    let (inferred_args, contextless) = inferred
        .map_or((String::from("unknown[]"), None), |inference| {
            (inference.arguments, inference.contextless_listener)
        });
    let fallthrough_box = contract.fallthrough_box.unwrap_or("[]");
    let loose = contextless.as_deref().unwrap_or("false");
    let helper = crate::virtual_ts::helpers::shared_types::LISTENER_CONTRACT.select(true);
    vize_l0::emit_ts!(
        ts,
        "{indent}type {alias} = {helper}<{library}, {props_box}, {emit_props_box}, {fallthrough_box}, {listener_key}, {raw_listener_key}, {inferred_args}, {loose}>;\n",
    );
    let script_binding = contract.has_script_component_binding;
    let typed_handler = contract.needs_typed_handler_assignment
        && (script_binding
            || requires_unresolved_handler_implicit_any(
                contract.summary,
                contract.component_name,
                contract.data,
                contract.scope,
            ));
    let handler_type_expr = typed_handler.then(|| {
        if script_binding {
            cstr!("{alias}[\"handler\"]")
        } else {
            cstr!("{alias}[\"looseHandler\"]")
        }
    });
    let handler_type = handler_type_expr
        .as_ref()
        .map(|_| cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_handler"));
    // Vue invokes every listener of an array; accept one only where the
    // child's own listener prop admits it, as vue-tsc.
    let handler_annotation = handler_type.as_ref().map(|handler| {
        if script_binding {
            cstr!("{handler} | {alias}[\"arrays\"]")
        } else {
            handler.clone()
        }
    });
    ComponentEventTypes {
        event_type: cstr!("{alias}[\"event\"]"),
        handler_type,
        handler_annotation,
        contextless_listener: contextless,
        handler_type_expr,
        listener_type: cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_listener"),
        listener_type_expr: cstr!("{alias}[\"listener\"]"),
    }
}
