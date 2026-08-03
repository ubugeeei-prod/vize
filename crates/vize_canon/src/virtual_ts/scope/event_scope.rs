//! The `EventHandler` scope closure: one typed wrapper per `@event` binding.
//!
//! A component `@event` gets the child's full emit-argument tuple as the
//! closure's rest parameter, so multi-argument emits keep every parameter
//! (#1512); a native DOM event gets the single `$event` its type implies.

use vize_carton::{String, append, profile};
use vize_croquis::{EventHandlerScopeData, Scope};

use crate::virtual_ts::helpers::get_dom_event_type;
use crate::virtual_ts::types::VizeMapping;

use super::component_events::generate_component_event_types;
use super::context::{EventHandlerExprContext, ScopeGenContext};
use super::event_handler::{event_name_source_range, generate_event_handler_expressions};

pub(super) fn generate_event_handler_scope(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ScopeGenContext<'_>,
    scope: &Scope,
    data: &EventHandlerScopeData,
    indent: &str,
    inner_indent: &str,
) {
    let scope_id = scope.id.as_u32();
    append!(*ts, "\n{indent}// @{} handler\n", data.event_name);

    if !ctx.check_options.check_emits {
        append!(*ts, "{indent}(($event: any) => {{\n");
        profile!(
            "canon.virtual_ts.event_handler_expressions",
            generate_event_handler_expressions(
                ts,
                mappings,
                scope_id,
                &EventHandlerExprContext {
                    expressions_by_scope: ctx.expressions_by_scope,
                    data,
                    check_emits: false,
                    event_type: "any",
                    event_handler_type: None,
                    event_listener_type: None,
                    event_name_src_range: None,
                    template_prop_names: ctx.template_prop_names,
                    template_offset: ctx.template_offset,
                    indent: inner_indent,
                },
            )
        );
        append!(*ts, "{indent}}})({{}} as any);\n");
        return;
    }

    if data.target_component.is_some() {
        let event_types = generate_component_event_types(
            ts,
            ctx.summary,
            data,
            scope,
            ctx.template_prop_names,
            ctx.legacy_vue2,
            indent,
        )
        .expect("component event handler should have a target component");
        let event_type = event_types.event_type;
        let handler_type = event_types.handler_type;
        let handler_type_expr = event_types.handler_type_expr;
        let listener_type = event_types.listener_type;
        let listener_type_expr = event_types.listener_type_expr;
        // Type the listener against the FULL emit tuple so multi-arg emits
        // keep every parameter (#1512); unresolved sigs stay variadic.
        append!(
            *ts,
            "{indent}type {listener_type} = {listener_type_expr};\n",
        );
        if let (Some(handler_type), Some(handler_type_expr)) = (&handler_type, &handler_type_expr) {
            append!(*ts, "{indent}type {handler_type} = {handler_type_expr};\n",);
        }
        // Receive listener args via a rest parameter typed by
        // `Parameters<listener>` to avoid TS2556; `$event` is element 0.
        append!(
            *ts,
            "{indent}((...__vize_args: Parameters<{listener_type}>) => {{\n",
        );
        append!(
            *ts,
            "{inner_indent}const $event = __vize_args[0] as {event_type}; void $event;\n",
        );

        profile!(
            "canon.virtual_ts.event_handler_expressions",
            generate_event_handler_expressions(
                ts,
                mappings,
                scope_id,
                &EventHandlerExprContext {
                    expressions_by_scope: ctx.expressions_by_scope,
                    data,
                    check_emits: true,
                    event_type: event_type.as_str(),
                    event_handler_type: handler_type.as_deref(),
                    event_listener_type: Some(listener_type.as_str()),
                    event_name_src_range: event_name_source_range(
                        ctx.template_source,
                        ctx.template_offset,
                        scope.span.start..scope.span.end,
                        data.event_name.as_str(),
                    ),
                    template_prop_names: ctx.template_prop_names,
                    template_offset: ctx.template_offset,
                    indent: inner_indent,
                },
            )
        );

        append!(
            *ts,
            "{indent}}})(...({{}} as Parameters<{listener_type}>));\n",
        );
    } else {
        let event_type = get_dom_event_type(data.event_name.as_str());
        append!(*ts, "{indent}(($event: {event_type}) => {{\n");

        profile!(
            "canon.virtual_ts.event_handler_expressions",
            generate_event_handler_expressions(
                ts,
                mappings,
                scope_id,
                &EventHandlerExprContext {
                    expressions_by_scope: ctx.expressions_by_scope,
                    data,
                    check_emits: true,
                    event_type,
                    event_handler_type: None,
                    // Native DOM listeners keep the identity-call shape,
                    // so there is no declared name to anchor at.
                    event_listener_type: None,
                    event_name_src_range: None,
                    template_prop_names: ctx.template_prop_names,
                    template_offset: ctx.template_offset,
                    indent: inner_indent,
                },
            )
        );

        append!(*ts, "{indent}}})({{}} as {event_type});\n");
    }
}
