//! Event-handler expression generation. The lightweight JS scanning that
//! classifies a handler body (callable reference vs. inline callback) lives in
//! [`super::handler_shape`].

use vize_carton::{String, append, cstr};

use crate::virtual_ts::expressions::rewrite_reserved_template_prop;
use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};

use super::context::EventHandlerExprContext;
use super::handler_shape::{inline_callback_event_argument, is_callable_handler_reference};
use super::vif_guard::append_ignored_vif_guard_open;

/// Generate event handler expressions inside a closure.
pub(super) fn generate_event_handler_expressions(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    scope_id: u32,
    ctx: &EventHandlerExprContext<'_>,
) {
    if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id) {
        for expr in exprs {
            let content = expr.content.as_str();
            let is_callable_reference = is_callable_handler_reference(content);
            let is_implicit_reference =
                ctx.check_emits && ctx.data.has_implicit_event && is_callable_reference;
            let inline_callback_arg = inline_callback_event_argument(content);
            let src_start = (ctx.template_offset + expr.start) as usize;
            let src_end = (ctx.template_offset + expr.end) as usize;
            let guard = expr.vif_guard.as_ref().map(|guard| {
                let trimmed_guard = guard.as_str().trim();
                rewrite_reserved_template_prop(trimmed_guard, ctx.template_prop_names)
                    .unwrap_or_else(|| String::from(guard.as_str()))
            });
            if let Some(ref guard) = guard {
                append_ignored_vif_guard_open(ts, ctx.indent, guard, "Inference-only guard");
            }
            let handler_indent = if guard.is_some() {
                cstr!("{}  ", ctx.indent)
            } else {
                String::from(ctx.indent)
            };

            // Component `@event` handlers carry the full emit listener type so
            // multi-arg emits keep every parameter (#1512). Both a bare callable
            // reference and an inline arrow/function are checked against the
            // listener type and invoked through the typed const with the full
            // argument spread. `__vize_args` is `Parameters<listener>` (a tuple),
            // so the spread always targets the listener's own parameter list,
            // verifying each parameter while avoiding TS2556.
            //
            // The listener type is an *annotation* on the const rather than the
            // parameter of an immediately-applied identity function: vue-tsc
            // assigns the handler to the child's `onEvent` prop and reports
            // `TS2322` at the `@event` attribute name, while a call reports
            // `TS2345` at the argument. TypeScript anchors a variable
            // declaration's assignability error at the declared name, so the
            // synthetic identifier is what maps back to the attribute (#3462).
            let mut listener_spans = None;
            let gen_range = if !ctx.check_emits
                && (is_callable_reference || inline_callback_arg.is_some())
            {
                append!(*ts, "{indent}void (", indent = handler_indent);
                let mapped_start = ts.len();
                ts.push_str(content);
                let mapped_end = ts.len();
                ts.push_str(");  // handler expression (emit checks disabled)\n");
                mapped_start..mapped_end
            } else if !ctx.check_emits {
                append!(*ts, "{indent}", indent = handler_indent);
                let mapped_start = ts.len();
                ts.push_str(content);
                let mapped_end = ts.len();
                ts.push_str(";  // handler expression (emit checks disabled)\n");
                mapped_start..mapped_end
            } else if let (Some(handler_type), Some(listener_type)) =
                (ctx.event_handler_type, ctx.event_listener_type)
                && (is_implicit_reference || inline_callback_arg.is_some())
            {
                let handler_name = cstr!("__vize_handler_{scope_id}_{}", expr.start);
                let stmt_start = ts.len();
                append!(*ts, "{indent}const ", indent = handler_indent);
                let name_start = ts.len();
                ts.push_str(handler_name.as_str());
                let name_end = ts.len();
                append!(*ts, ": {handler_type} | null | undefined = (");
                let mapped_start = ts.len();
                ts.push_str(content);
                let mapped_end = ts.len();
                ts.push_str(");\n");
                listener_spans = Some((stmt_start..ts.len(), name_start..name_end));
                append!(
                    *ts,
                    "{indent}if (typeof {handler_name} === \"function\") ({handler_name} as {listener_type})(...__vize_args);  // handler expression\n",
                    indent = handler_indent,
                );
                mapped_start..mapped_end
            } else if let Some(listener_type) = ctx.event_listener_type
                && (is_implicit_reference || inline_callback_arg.is_some())
            {
                let handler_name = cstr!("__vize_handler_{scope_id}_{}", expr.start);
                let stmt_start = ts.len();
                append!(*ts, "{indent}const ", indent = handler_indent);
                let name_start = ts.len();
                ts.push_str(handler_name.as_str());
                let name_end = ts.len();
                append!(*ts, ": {listener_type} | null | undefined = (");
                let mapped_start = ts.len();
                ts.push_str(content);
                let mapped_end = ts.len();
                ts.push_str(");\n");
                listener_spans = Some((stmt_start..ts.len(), name_start..name_end));
                append!(
                    *ts,
                    "{indent}if ({handler_name}) {handler_name}(...__vize_args);  // handler expression\n",
                    indent = handler_indent,
                );
                mapped_start..mapped_end
            } else if is_implicit_reference {
                let handler_name = cstr!("__vize_handler_{scope_id}_{}", expr.start);
                append!(
                    *ts,
                    "{indent}const {handler_name} = ((__vize_cb: ((_e: {event_type}) => unknown) | null | undefined) => __vize_cb)((",
                    indent = handler_indent,
                    event_type = ctx.event_type,
                );
                let mapped_start = ts.len();
                ts.push_str(content);
                let mapped_end = ts.len();
                ts.push_str("));\n");
                append!(
                    *ts,
                    "{indent}if ({handler_name}) {handler_name}($event);  // handler expression\n",
                    indent = handler_indent,
                );
                mapped_start..mapped_end
            } else if let Some(event_arg) = inline_callback_arg {
                // Wrap the inline callback invocation in a closure that
                // re-declares `$event` typed against the handler's event type.
                // The outer EventHandler closure already binds `$event`, but
                // this inner wrap pins the binding immediately around the
                // user's callback so the inline arrow body can reference
                // `$event` directly (#2224 — `Cannot find name '$event'`).
                append!(
                    *ts,
                    "{indent}(($event: {event_type}) => {{ (",
                    indent = handler_indent,
                    event_type = ctx.event_type,
                );
                let mapped_start = ts.len();
                ts.push_str(content);
                let mapped_end = ts.len();
                append!(*ts, ")({event_arg}); }})($event);  // handler expression\n");
                mapped_start..mapped_end
            } else {
                append!(*ts, "{indent}", indent = handler_indent);
                let mapped_start = ts.len();
                ts.push_str(content);
                let mapped_end = ts.len();
                ts.push_str(";  // handler expression\n");
                mapped_start..mapped_end
            };
            // The declared identifier receives the handler-shape error, which
            // vue-tsc anchors at the attribute name; the initializer keeps the
            // authored expression so errors inside an inline callback still land
            // on the authored bytes. The two sub-spans replace the mapping's own
            // range entirely, so the mapping has to widen to the whole statement
            // for the identifier to be inside it — the diagnostic path looks the
            // mapping up by `gen_range` before narrowing to a sub-span.
            // When the event name cannot be located in the source the bound
            // expression is the next best anchor, so the identifier's error
            // still maps back instead of being dropped for want of a sub-span
            // (same fallback as a component prop's name/value ranges).
            let (gen_range, sub_spans) = match listener_spans {
                Some((stmt_gen_range, name_gen_range)) => (
                    stmt_gen_range,
                    vec![
                        VizeSubSpan {
                            gen_range: name_gen_range,
                            src_range: ctx
                                .event_name_src_range
                                .clone()
                                .unwrap_or(src_start..src_end),
                        },
                        VizeSubSpan {
                            gen_range,
                            src_range: src_start..src_end,
                        },
                    ],
                ),
                None => (gen_range, Vec::new()),
            };
            mappings.push(VizeMapping {
                gen_range,
                src_range: src_start..src_end,
                sub_spans,
            });
            append!(
                *ts,
                "{indent}// @vize-map: handler -> {src_start}:{src_end}\n",
                indent = handler_indent,
            );
            if guard.is_some() {
                append!(*ts, "{indent}}}\n", indent = ctx.indent);
            }
        }
    }
}

/// The authored range of the event name inside an `@event` / `v-on:event`
/// attribute, given the directive's template-relative `span`.
///
/// vue-tsc anchors a wrongly-shaped handler at that name, not at the `@` and
/// not at the bound expression. Both spellings are validated against the
/// source: the directive text must start with the prefix and the event name
/// must follow it, so a span that is not the directive (or a template that
/// cannot be read back) yields `None` and the mapping falls back to the
/// expression.
pub(super) fn event_name_source_range(
    template_source: Option<&str>,
    template_offset: u32,
    span: std::ops::Range<u32>,
    event_name: &str,
) -> Option<std::ops::Range<usize>> {
    let directive = template_source?.get(span.start as usize..span.end as usize)?;
    let prefix_len = ["@", "v-on:"].into_iter().find_map(|prefix| {
        directive
            .strip_prefix(prefix)
            .filter(|rest| rest.starts_with(event_name))
            .map(|_| prefix.len())
    })?;
    let start = template_offset as usize + span.start as usize + prefix_len;
    Some(start..start + event_name.len())
}
