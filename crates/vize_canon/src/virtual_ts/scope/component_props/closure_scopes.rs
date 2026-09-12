use vize_carton::{String, append, cstr, profile};
use vize_croquis::{Scope, ScopeData, ScopeKind};

use crate::virtual_ts::types::VizeMapping;

use super::super::context::VForPropsContext;
use super::super::emit::{append_v_for_comment, emit_v_for_loop_open};
use super::super::empty_component_props::generate_scope_checks;
use super::super::slot_scope::generate_v_slot_props_scope;

pub(super) fn generate_closure_component_props_recursive(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &VForPropsContext<'_, '_>,
    scope: &Scope,
    indent: &str,
) {
    let scope_id = scope.id.as_u32();
    let inner_indent = cstr!("{indent}  ");
    match scope.data() {
        ScopeData::VFor(data) => {
            let enclosing_guard = ctx.vfor_enclosing_guards.get(&scope_id).map(String::as_str);
            let (loop_indent, vfor_inner_indent) = if enclosing_guard.is_some() {
                (cstr!("{indent}  "), cstr!("{inner_indent}  "))
            } else {
                (String::from(indent), inner_indent.clone())
            };
            if let Some(guard) = enclosing_guard {
                append!(*ts, "{indent}if ({guard}) {{\n");
            }

            append_v_for_comment(
                ts,
                &loop_indent,
                "Component props in v-for scope",
                data.value_alias.as_str(),
                data.source.as_str(),
            );
            emit_v_for_loop_open(
                ts,
                mappings,
                ctx.source_context.offset,
                ctx.summary.scopes.v_for_source_offset(scope.id),
                &loop_indent,
                scope,
                ctx.template_prop_names,
            );

            for value in &data.value_bindings {
                append!(*ts, "{vfor_inner_indent}void {value};\n");
            }
            if let Some(ref key) = data.key_alias {
                append!(*ts, "{vfor_inner_indent}void {key};\n");
            }
            if let Some(ref index) = data.index_alias {
                append!(*ts, "{vfor_inner_indent}void {index};\n");
            }

            generate_scope_checks(ts, mappings, ctx, scope_id, &vfor_inner_indent);
            recurse_child_closure_scopes(ts, mappings, ctx, scope_id, &vfor_inner_indent);

            ts.push_str(&loop_indent);
            ts.push_str("});\n");
            if enclosing_guard.is_some() {
                append!(*ts, "{indent}}}\n");
            }
        }
        ScopeData::VSlot(data) => {
            generate_v_slot_props_scope(ts, mappings, ctx, scope, data, indent, &inner_indent);
        }
        _ => {}
    }
}

pub(in crate::virtual_ts::scope) fn recurse_child_closure_scopes(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &VForPropsContext<'_, '_>,
    scope_id: u32,
    indent: &str,
) {
    let Some(child_ids) = ctx.children_map.get(&scope_id) else {
        return;
    };
    for &child_id in child_ids {
        if let Some(child_scope) = ctx.summary.scopes.get_scope(child_id)
            && matches!(child_scope.kind, ScopeKind::VFor | ScopeKind::VSlot)
        {
            profile!(
                "canon.virtual_ts.closure_component_props",
                generate_closure_component_props_recursive(ts, mappings, ctx, child_scope, indent)
            );
        }
    }
}
