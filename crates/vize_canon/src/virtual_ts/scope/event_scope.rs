//! The `EventHandler` scope closure: one typed wrapper per `@event` binding.
//!
//! A component `@event` gets the child's full emit-argument tuple as the
//! closure's rest parameter, so multi-argument emits keep every parameter
//! (#1512); a native DOM event gets the single `$event` its type implies.
//!
//! Every wrapper is referenced with `void`, never invoked: TypeScript inlines
//! an immediately-invoked function expression's body into the enclosing
//! control-flow graph, so an inline handler assignment (`@click="x = 'a'"`)
//! would narrow `x` for every sibling binding checked after it — the false
//! TS2367 of #4962. A handler is a runtime callback; a plain function
//! expression keeps its body fully checked while its assignments stay out of
//! the render scope's flow, matching `vue-tsc`. Narrowing INTO a handler does
//! not rely on IIFE inlining either: each handler expression re-checks its own
//! `v-if` guard inside the closure (see `event_handler.rs`).

use crate::virtual_ts::scope::ScopeMappings;
use vize_carton::{append, profile};
use vize_croquis::{EventHandlerScopeData, Scope};

use crate::virtual_ts::helpers::get_dom_event_type;

mod component_scope;
mod event_targets;

use super::context::{EventHandlerExprContext, ScopeGenContext};
use super::event_handler::{event_name_source_range, generate_event_handler_expressions};
use event_targets::{
    dynamic_component_custom_event, needs_typed_handler_assignment, transition_hook_signature,
    vnode_hook_signature,
};

#[derive(Clone, Copy)]
pub(super) struct EventVifGuards<'a> {
    pub(super) handler: Option<&'a str>,
    pub(super) active_outer: Option<&'a str>,
}

pub(super) fn generate_event_handler_scope(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut ScopeMappings<'_>,
    ctx: &ScopeGenContext<'_, '_>,
    scope: &Scope,
    data: &EventHandlerScopeData,
    vif_guards: EventVifGuards<'_>,
    indents: (&str, &str),
) {
    let (indent, inner_indent) = indents;
    let scope_id = scope.id.as_u32();
    let event_value = if data.has_implicit_event {
        "$event"
    } else {
        "__vize_event"
    };
    append!(*ts, "\n{indent}// @{} handler\n", data.event_name);

    // An inline body that never reads its implicit argument needs only a
    // function boundary. Keep assignments out of render control flow without
    // instantiating unused DOM event types. A component listener is the
    // exception: its body is still the handler the child's listener prop
    // receives, so it is declared against that prop's type below even when
    // `$event` goes unread.
    let unused_event = data.has_implicit_event
        && scope
            .get_binding("$event")
            .is_some_and(|binding| !binding.is_used());
    if !ctx.check_options.check_emits
        || (unused_event && data.target_component.is_none() && !data.event_name_is_dynamic)
    {
        if unused_event {
            vize_l0::emit_ts!(ts, "{indent}void (() => {{\n");
        } else {
            vize_l0::emit_ts!(ts, "{indent}void (({event_value}: any) => {{\n");
        }
        profile!(
            "canon.virtual_ts.event_handler_expressions",
            generate_event_handler_expressions(
                ts,
                mappings,
                scope_id,
                &EventHandlerExprContext {
                    expressions_by_scope: ctx.expressions_by_scope,
                    diagnostic_ownership: ctx.checks.diagnostic_ownership,
                    data,
                    check_emits: ctx.check_options.check_emits,
                    event_type: "any",
                    event_handler_contextless: None,
                    event_handler_type: None,
                    event_listener_type: None,
                    event_name_src_range: None,
                    return_single_expression: false,
                    enclosing_vif_guard: vif_guards.handler,
                    template_binding_access: ctx.template_binding_access,
                    template_offset: ctx.template_offset,
                    indent: inner_indent,
                },
            )
        );
        vize_l0::emit_ts!(ts, "{indent}}});\n");
        return;
    }

    if let Some((event_type, listener_args)) = (!data.event_name_is_dynamic)
        .then(|| vnode_hook_signature(data.event_name.as_str()))
        .flatten()
    {
        let listener_type = vize_carton::cstr!("__vize_vnode_hook_{}_listener", scope_id);
        let handler_type = vize_carton::cstr!("__vize_vnode_hook_{}_handler", scope_id);
        let needs_typed_handler_assignment = needs_typed_handler_assignment(data);
        vize_l0::emit_ts!(
            ts,
            "{indent}type {listener_type} = (...args: {listener_args}) => any;\n"
        );
        if needs_typed_handler_assignment {
            vize_l0::emit_ts!(
                ts,
                "{indent}type {handler_type} = {{ bivarianceHack(...args: {listener_args}): any }}[\"bivarianceHack\"];\n"
            );
        }
        vize_l0::emit_ts!(
            ts,
            "{indent}void ((...__vize_args: Parameters<{listener_type}>) => {{\n"
        );
        vize_l0::emit_ts!(
            ts,
            "{inner_indent}const {event_value} = __vize_args[0] as {event_type}; void {event_value};\n"
        );

        profile!(
            "canon.virtual_ts.event_handler_expressions",
            generate_event_handler_expressions(
                ts,
                mappings,
                scope_id,
                &EventHandlerExprContext {
                    expressions_by_scope: ctx.expressions_by_scope,
                    diagnostic_ownership: ctx.checks.diagnostic_ownership,
                    data,
                    check_emits: true,
                    event_type,
                    event_handler_contextless: None,
                    event_handler_type: needs_typed_handler_assignment
                        .then_some(handler_type.as_str()),
                    event_listener_type: Some(listener_type.as_str()),
                    event_name_src_range: event_name_source_range(
                        ctx.template_source,
                        ctx.template_offset,
                        scope.span.start..scope.span.end,
                        data.event_name.as_str(),
                    ),
                    return_single_expression: false,
                    enclosing_vif_guard: vif_guards.handler,
                    template_binding_access: ctx.template_binding_access,
                    template_offset: ctx.template_offset,
                    indent: inner_indent,
                },
            )
        );

        vize_l0::emit_ts!(ts, "{indent}}});\n");
    } else if data.target_component.is_some() && !data.event_name_is_dynamic {
        component_scope::generate_component_handler_scope(
            ts,
            mappings,
            ctx,
            scope,
            data,
            vif_guards,
            (indent, inner_indent),
        );
    } else if let Some((event_type, listener_args)) = (!data.event_name_is_dynamic)
        .then(|| {
            transition_hook_signature(
                ctx.template_source,
                ctx.template_ast,
                scope.span.start,
                data.event_name.as_str(),
            )
        })
        .flatten()
    {
        let listener_type = vize_carton::cstr!("__vize_transition_{}_listener", scope_id);
        let handler_type = vize_carton::cstr!("__vize_transition_{}_handler", scope_id);
        let needs_typed_handler_assignment = needs_typed_handler_assignment(data);
        vize_l0::emit_ts!(
            ts,
            "{indent}type {listener_type} = (...args: {listener_args}) => any;\n"
        );
        if needs_typed_handler_assignment {
            vize_l0::emit_ts!(
                ts,
                "{indent}type {handler_type} = {{ bivarianceHack(...args: {listener_args}): any }}[\"bivarianceHack\"];\n"
            );
        }
        vize_l0::emit_ts!(
            ts,
            "{indent}void ((...__vize_args: Parameters<{listener_type}>) => {{\n"
        );
        vize_l0::emit_ts!(
            ts,
            "{inner_indent}const {event_value} = __vize_args[0] as {event_type}; void {event_value};\n"
        );

        profile!(
            "canon.virtual_ts.event_handler_expressions",
            generate_event_handler_expressions(
                ts,
                mappings,
                scope_id,
                &EventHandlerExprContext {
                    expressions_by_scope: ctx.expressions_by_scope,
                    diagnostic_ownership: ctx.checks.diagnostic_ownership,
                    data,
                    check_emits: true,
                    event_type,
                    event_handler_contextless: None,
                    event_handler_type: needs_typed_handler_assignment
                        .then_some(handler_type.as_str()),
                    event_listener_type: Some(listener_type.as_str()),
                    event_name_src_range: event_name_source_range(
                        ctx.template_source,
                        ctx.template_offset,
                        scope.span.start..scope.span.end,
                        data.event_name.as_str(),
                    ),
                    return_single_expression: false,
                    enclosing_vif_guard: vif_guards.handler,
                    template_binding_access: ctx.template_binding_access,
                    template_offset: ctx.template_offset,
                    indent: inner_indent,
                },
            )
        );

        vize_l0::emit_ts!(ts, "{indent}}});\n");
    } else if data.event_name_is_dynamic
        || dynamic_component_custom_event(
            ctx.template_source,
            ctx.template_ast,
            scope.span.start,
            data.event_name.as_str(),
        )
    {
        let listener_type = vize_carton::cstr!("__vize_dynamic_component_{}_listener", scope_id);
        vize_l0::emit_ts!(
            ts,
            "{indent}type {listener_type} = (...args: any[]) => any;\n"
        );
        vize_l0::emit_ts!(
            ts,
            "{indent}void ((...__vize_args: Parameters<{listener_type}>) => {{\n"
        );
        vize_l0::emit_ts!(
            ts,
            "{inner_indent}const {event_value} = __vize_args[0] as any; void {event_value};\n"
        );

        profile!(
            "canon.virtual_ts.event_handler_expressions",
            generate_event_handler_expressions(
                ts,
                mappings,
                scope_id,
                &EventHandlerExprContext {
                    expressions_by_scope: ctx.expressions_by_scope,
                    diagnostic_ownership: ctx.checks.diagnostic_ownership,
                    data,
                    check_emits: true,
                    event_type: "any",
                    event_handler_contextless: None,
                    event_handler_type: None,
                    event_listener_type: Some(listener_type.as_str()),
                    event_name_src_range: event_name_source_range(
                        ctx.template_source,
                        ctx.template_offset,
                        scope.span.start..scope.span.end,
                        data.event_name.as_str(),
                    ),
                    return_single_expression: false,
                    enclosing_vif_guard: vif_guards.handler,
                    template_binding_access: ctx.template_binding_access,
                    template_offset: ctx.template_offset,
                    indent: inner_indent,
                },
            )
        );

        vize_l0::emit_ts!(ts, "{indent}}});\n");
    } else {
        let event_type = get_dom_event_type(data.event_name.as_str());
        vize_l0::emit_ts!(ts, "{indent}void (({event_value}: {event_type}) => {{\n");

        profile!(
            "canon.virtual_ts.event_handler_expressions",
            generate_event_handler_expressions(
                ts,
                mappings,
                scope_id,
                &EventHandlerExprContext {
                    expressions_by_scope: ctx.expressions_by_scope,
                    diagnostic_ownership: ctx.checks.diagnostic_ownership,
                    data,
                    check_emits: true,
                    event_type,
                    event_handler_contextless: None,
                    event_handler_type: None,
                    // Native DOM listeners keep the plain-closure shape,
                    // so there is no declared name to anchor at.
                    event_listener_type: None,
                    event_name_src_range: None,
                    return_single_expression: false,
                    enclosing_vif_guard: vif_guards.handler,
                    template_binding_access: ctx.template_binding_access,
                    template_offset: ctx.template_offset,
                    indent: inner_indent,
                },
            )
        );

        vize_l0::emit_ts!(ts, "{indent}}});\n");
    }
}
