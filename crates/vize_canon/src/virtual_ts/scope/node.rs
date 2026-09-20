use super::children::generate_child_scopes;
use super::context::ScopeGenContext;
use super::emit::{append_v_for_comment, emit_v_for_loop_open};
use super::event_scope::generate_event_handler_scope;
use super::slot_outlet_props::generate_scope_slot_outlet_checks;
use super::slot_scope::generate_v_slot_scope;
use super::vif_guard::{
    append_ignored_vif_guard_open, callback_vif_guard, common_vif_guard_prefix_outside_v_for_scope,
};
use crate::virtual_ts::expressions::{
    ExpressionListEmitContext, generate_expressions, generate_expressions_in_enclosing_guard,
};
use crate::virtual_ts::types::VizeMapping;
use vize_carton::{String, append, cstr, profile};
use vize_croquis::{Scope, ScopeData};

pub(super) fn generate_scope_node(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ScopeGenContext<'_, '_>,
    scope: &Scope,
    indent: &str,
) {
    let scope_id = scope.id.as_u32();
    let inner_indent = cstr!("{indent}  ");

    match scope.data() {
        ScopeData::VMatch(_) => {
            super::patterns::generate_expression_match(ts, mappings, ctx, scope, indent)
        }
        ScopeData::VFor(data) => {
            let capture_slots = ctx.slot_outlets.captures_scope(ctx.summary, scope.id);
            // Re-emit parent `v-if` around v-for source so TypeScript keeps narrowing (#1511).
            let enclosing_guard: Option<String> = ctx
                .expressions_by_scope
                .get(&scope_id)
                .filter(|_| ctx.check_options.check_template_bindings)
                .and_then(|exprs| common_vif_guard_prefix_outside_v_for_scope(exprs, scope));
            let enclosing_guard = enclosing_guard.as_deref();
            let (loop_indent, vfor_inner_indent) = if enclosing_guard.is_some() {
                (cstr!("{indent}  "), cstr!("{inner_indent}  "))
            } else {
                (String::from(indent), inner_indent.clone())
            };
            if let Some(guard) = enclosing_guard {
                append_ignored_vif_guard_open(ts, indent, guard, "Narrowing-only guard");
            }
            append_v_for_comment(
                ts,
                &loop_indent,
                "v-for scope",
                data.value_alias.as_str(),
                data.source.as_str(),
            );
            emit_v_for_loop_open(
                ts,
                mappings,
                ctx.template_offset,
                ctx.summary.scopes.v_for_source_offset(scope.id),
                &loop_indent,
                scope,
                ctx.template_binding_access,
                capture_slots,
            );
            // Recheck positive terms for callback-captured object-property narrowing.
            let callback_guard = enclosing_guard
                .filter(|_| capture_slots)
                .and_then(callback_vif_guard);
            let callback_indent = if let Some(guard) = callback_guard.as_deref() {
                append_ignored_vif_guard_open(
                    ts,
                    vfor_inner_indent.as_str(),
                    guard,
                    "Narrowing-only guard",
                );
                cstr!("{vfor_inner_indent}  ")
            } else {
                vfor_inner_indent.clone()
            };

            // Mark v-for variables as used to avoid TS6133
            for value in &data.value_bindings {
                append!(*ts, "{callback_indent}void {value};\n");
            }
            if let Some(ref key) = data.key_alias {
                append!(*ts, "{callback_indent}void {key};\n");
            }
            if let Some(ref index) = data.index_alias {
                append!(*ts, "{callback_indent}void {index};\n");
            }

            if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id)
                && ctx.check_options.check_template_bindings
            {
                generate_expressions_in_enclosing_guard(
                    ts,
                    mappings,
                    exprs,
                    ctx.template_binding_access,
                    &ExpressionListEmitContext::new(
                        ctx.skipped_expression_ranges,
                        ctx.template_offset,
                        &callback_indent,
                        ctx.checks,
                    ),
                    enclosing_guard,
                );
            }
            generate_scope_slot_outlet_checks(ts, mappings, scope_id, ctx, &callback_indent);

            profile!(
                "canon.virtual_ts.child_scopes",
                generate_child_scopes(ts, mappings, ctx, scope_id, &callback_indent)
            );
            if capture_slots {
                ctx.slot_outlets
                    .emit_result(ts, ctx.summary, Some(scope.id), &callback_indent);
            }

            if callback_guard.is_some() {
                append!(*ts, "{vfor_inner_indent}}}\n");
            }

            ts.push_str(&loop_indent);
            ts.push_str("}\n");
            if capture_slots {
                append!(*ts, "{loop_indent}return [];\n{loop_indent}}})();\n");
            }

            append!(*ts, "{loop_indent}}}\n");
            if enclosing_guard.is_some() {
                append!(*ts, "{indent}}}\n");
            }
        }
        ScopeData::VSlot(data) => {
            generate_v_slot_scope(ts, mappings, ctx, scope, data, indent, &inner_indent);
        }
        ScopeData::EventHandler(_)
            if ctx
                .slot_outlets
                .covers_event_handler_scope(scope.span.start, scope.span.end) => {}
        ScopeData::EventHandler(data) if ctx.check_options.check_event_handlers() => {
            generate_event_handler_scope(ts, mappings, ctx, scope, data, indent, &inner_indent);
        }
        _ => {
            generate_scope_contents(ts, mappings, ctx, scope, indent);
            if matches!(scope.data(), ScopeData::VWhen(_)) {
                generate_child_scopes(ts, mappings, ctx, scope_id, indent);
            }
        }
    }
}

pub(super) fn generate_scope_contents(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ScopeGenContext<'_, '_>,
    scope: &Scope,
    indent: &str,
) {
    let scope_id = scope.id.as_u32();
    if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id)
        && ctx.check_options.check_template_bindings
    {
        generate_expressions(
            ts,
            mappings,
            exprs,
            ctx.template_binding_access,
            &ExpressionListEmitContext::new(
                ctx.skipped_expression_ranges,
                ctx.template_offset,
                indent,
                ctx.checks,
            ),
        );
    }
    generate_scope_slot_outlet_checks(ts, mappings, scope_id, ctx, indent);
}
