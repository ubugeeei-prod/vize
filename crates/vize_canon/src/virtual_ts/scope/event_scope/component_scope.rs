//! The component `@event` closure: the child's full emit-argument tuple is
//! the closure's rest parameter, and an inline statement body is declared
//! against the child's listener prop type so its return value is checked.

use vize_carton::{String, append, profile};
use vize_croquis::{EventHandlerScopeData, Scope};

use crate::virtual_ts::types::VizeMapping;

use super::super::component_events::{ComponentEventTypeContext, generate_component_event_types};
use super::super::context::{EventHandlerExprContext, ScopeGenContext};
use super::super::event_handler::{event_name_source_range, generate_event_handler_expressions};
use super::event_targets::needs_typed_handler_assignment;

pub(super) fn generate_component_handler_scope(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ScopeGenContext<'_, '_>,
    scope: &Scope,
    data: &EventHandlerScopeData,
    indent: &str,
    inner_indent: &str,
) {
    let scope_id = scope.id.as_u32();
    let event_value = if data.has_implicit_event {
        "$event"
    } else {
        "__vize_event"
    };
    let needs_typed_handler_assignment = needs_typed_handler_assignment(data);
    // Only reached for handlers with a target component; without one there is
    // no event contract to declare.
    let Some(event_types) = generate_component_event_types(
        ts,
        ComponentEventTypeContext {
            summary: ctx.summary,
            virtual_ts_options: ctx.virtual_ts_options,
            data,
            scope,
            syntactic_type_only_imported_names: ctx.syntactic_type_only_imported_names,
            template_binding_access: ctx.template_binding_access,
            legacy_vue2: ctx.legacy_vue2,
            needs_typed_handler_assignment,
            fallthrough_listeners: ctx.check_options.fallthrough_attributes,
            explicit_generic: data.target_component.as_ref().and_then(|component| {
                ctx.explicit_generics
                    .alias_at(ctx.summary, component.as_str(), scope.span.start)
            }),
            indent,
        },
    ) else {
        return;
    };
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
    let event_name_src_range = event_name_source_range(
        ctx.template_source,
        ctx.template_offset,
        scope.span.start..scope.span.end,
        data.event_name.as_str(),
    );
    // Keep the handler body in a function scope so assignments do not
    // narrow the surrounding render scope; `$event` is element 0.
    //
    // An inline statement body is the handler `vue-tsc` synthesizes for
    // the listener prop, so it is declared against the listener type
    // rather than merely referenced: a lone expression is returned (its
    // value is the handler's result) and anything else returns nothing,
    // which a prop typed `onSave?: () => number` rejects. The declared
    // name anchors that rejection at the authored event name (#3462).
    let inline_statement_handler = data.has_implicit_event && !ctx.legacy_vue2;
    let mut inline_name_range = None;
    if inline_statement_handler {
        let handler_name = vize_carton::cstr!("__vize_inline_handler_{}", scope_id);
        append!(*ts, "{indent}const ");
        let name_start = ts.len();
        ts.push_str(handler_name.as_str());
        inline_name_range = Some(name_start..ts.len());
        append!(
            *ts,
            ": {listener_type} = (...__vize_args: Parameters<{listener_type}>) => {{\n",
        );
    } else {
        append!(
            *ts,
            "{indent}void ((...__vize_args: Parameters<{listener_type}>) => {{\n",
        );
    }
    append!(
        *ts,
        "{inner_indent}const {event_value} = __vize_args[0] as {event_type}; void {event_value};\n",
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
                event_name_src_range: event_name_src_range.clone(),
                return_single_expression: inline_statement_handler,
                template_binding_access: ctx.template_binding_access,
                template_offset: ctx.template_offset,
                indent: inner_indent,
            },
        )
    );

    if let Some(name_range) = inline_name_range {
        append!(
            *ts,
            "{indent}}};\n{indent}void __vize_inline_handler_{scope_id};\n"
        );
        let src_range = event_name_src_range.unwrap_or(
            (ctx.template_offset + scope.span.start) as usize
                ..(ctx.template_offset + scope.span.end) as usize,
        );
        mappings.push(VizeMapping {
            gen_range: name_range,
            src_range,
            sub_spans: Vec::new(),
        });
    } else {
        append!(*ts, "{indent}}});\n");
    }
}
