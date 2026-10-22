use vize_carton::{String, append, cstr, profile};
use vize_croquis::{Scope, ScopeData, ScopeKind};

use crate::virtual_ts::types::VizeMapping;

use super::super::context::VForPropsContext;
use super::super::emit::{append_v_for_comment, emit_v_for_loop_open};
use super::super::empty_component_props::generate_scope_checks;
use super::super::slot_scope::generate_v_slot_props_scope;

pub(in crate::virtual_ts::scope) fn generate_closure_component_props_recursive(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &VForPropsContext<'_, '_>,
    scope: &Scope,
    indent: &str,
) {
    let scope_id = scope.id.as_u32();
    let inner_indent = cstr!("{indent}  ");
    match scope.data() {
        ScopeData::VMatch(_) => {
            super::super::patterns::generate_props_match(ts, mappings, ctx, scope, indent)
        }
        ScopeData::VWhen(_) => {
            generate_scope_checks(ts, mappings, ctx, scope_id, indent);
            recurse_child_closure_scopes(ts, mappings, ctx, scope_id, indent);
        }
        ScopeData::VFor(data) => {
            let enclosing_guard = ctx.vfor_enclosing_guards.get(&scope_id).map(String::as_str);
            let emitted_guard =
                enclosing_guard.filter(|guard| !v_for_ancestor_owns_guard(ctx, scope, guard));
            let (loop_indent, vfor_inner_indent) = if emitted_guard.is_some() {
                (cstr!("{indent}  "), cstr!("{inner_indent}  "))
            } else {
                (String::from(indent), inner_indent.clone())
            };
            if let Some(guard) = emitted_guard {
                super::super::vif_guard_chain::append_bare_chained_vif_guard_open(
                    ts, indent, guard,
                );
            }
            let suspended = emitted_guard
                .is_some()
                .then(super::super::vif_guard_chain::SuspendedChain::suspend);

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
                ctx.template_binding_access,
                false,
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

            {
                let prefix =
                    enclosing_guard.map(super::super::vif_guard_chain::ActiveGuardPrefix::push);
                generate_scope_checks(ts, mappings, ctx, scope_id, &vfor_inner_indent);
                drop(prefix);
            }
            recurse_child_closure_scopes(ts, mappings, ctx, scope_id, &vfor_inner_indent);

            ts.push_str(&loop_indent);
            ts.push_str("}\n");
            append!(*ts, "{loop_indent}}}\n");
            drop(suspended);
            if emitted_guard.is_some() {
                super::super::append_chained_vif_guard_close(ts, indent);
            }
        }
        ScopeData::VSlot(data) => {
            generate_v_slot_props_scope(ts, mappings, ctx, scope, data, indent, &inner_indent);
        }
        _ => {}
    }
}

/// A replayed child loop stays lexically inside every ancestor loop. Reopening
/// the same cumulative branch guard would compare an already narrowed
/// discriminant and can produce an unmapped TS2367 in Content Mapper hosts.
fn v_for_ancestor_owns_guard(ctx: &VForPropsContext<'_, '_>, scope: &Scope, guard: &str) -> bool {
    let mut parent = scope.parent();
    while let Some(parent_id) = parent {
        let Some(parent_scope) = ctx.summary.scopes.get_scope(parent_id) else {
            return false;
        };
        if matches!(parent_scope.kind, ScopeKind::VFor)
            && ctx
                .vfor_enclosing_guards
                .get(&parent_scope.id.as_u32())
                .is_some_and(|parent_guard| parent_guard == guard)
        {
            return true;
        }
        parent = parent_scope.parent();
    }
    false
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
            && matches!(
                child_scope.kind,
                ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
            )
        {
            profile!(
                "canon.virtual_ts.closure_component_props",
                generate_closure_component_props_recursive(ts, mappings, ctx, child_scope, indent)
            );
        }
    }
}
