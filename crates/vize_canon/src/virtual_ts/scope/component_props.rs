//! Scope-aware component props type checks, including recursion into nested
//! v-for/v-slot closure scopes.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;

use vize_croquis::{Croquis, Scope, ScopeData, ScopeKind, analysis::ComponentUsage};

use crate::virtual_ts::expressions::generate_component_prop_checks;
use crate::virtual_ts::helpers::{to_safe_identifier, to_safe_identifier_fragment};
use crate::virtual_ts::types::VizeMapping;

use super::component_prop_checker::{
    append_per_prop_aliases, append_prop_check_helpers, append_prop_checker_alias,
};
use super::component_prop_navigation;
use super::context::{ComponentPropsContext, GlobalComponentCheck, VForPropsContext};
use super::emit::{
    append_v_for_comment, emit_slot_function_open, emit_v_for_loop_open, slot_props_type,
};
use super::empty_component_props::{
    generate_empty_root_checks, generate_scope_checks, is_empty_props_usage,
};
use super::vif_guard::common_vif_guard_prefix_for_guards_outside_v_for;

/// Generate component props type checks (scope-aware).
/// Type declarations are at template level, value checks are in their scope.
pub(super) fn generate_component_props(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_>,
    checkable_usages: &[(usize, &ComponentUsage)],
) {
    let summary = ctx.summary;
    if checkable_usages.is_empty() {
        return;
    }

    let mut components_by_scope: FxHashMap<u32, Vec<(usize, &ComponentUsage)>> =
        FxHashMap::default();
    for &(idx, usage) in checkable_usages {
        components_by_scope
            .entry(usage.scope_id.as_u32())
            .or_default()
            .push((idx, usage));
    }

    ts.push_str("\n  // Component props type declarations\n");

    // Generic children expose `__vizeCheck<T>(props)`; fallback contextual
    // typing is limited to inline function props to avoid duplicate errors.
    append_prop_check_helpers(ts, checkable_usages);

    for &(idx, usage) in checkable_usages {
        let component_ref = to_safe_identifier(usage.name.as_str());
        let component_type_name = to_safe_identifier_fragment(usage.name.as_str());

        let src_start = (ctx.template_offset + usage.start) as usize;
        let src_end = (ctx.template_offset + usage.end) as usize;
        append!(*ts, "  // @vize-map: component -> {src_start}:{src_end}\n",);
        // Prefer the modern static raw-props identity while retaining the
        // instance marker for declarations emitted before #4034.
        append!(
            *ts,
            "  type __{component_type_name}_Props_{idx} = typeof {component_ref} extends {{ __vizeCheck: any }} ? Record<string, unknown> : (typeof {component_ref} extends {{ readonly __vizeRawProps?: infer __P }} ? __P : (typeof {component_ref} extends {{ new (): {{ readonly __vizeRawProps?: infer __P }} }} ? __P : (typeof {component_ref} extends {{ new (): {{ $props: infer __P }} }} ? __P : (typeof {component_ref} extends (props: infer __P) => any ? __P : {{}}))));\n",
        );

        append_per_prop_aliases(ts, usage, component_type_name.as_str(), idx);

        // Generic functional prop-checker for this component (#775).
        append_prop_checker_alias(
            ts,
            component_type_name.as_str(),
            component_ref.as_str(),
            idx,
        );
    }

    component_prop_navigation::emit_references(ts, mappings, ctx, checkable_usages);

    // Collect all closure scope IDs (v-for and v-slot)
    let closure_scope_ids: FxHashSet<u32> = summary
        .scopes
        .iter()
        .filter(|s| matches!(s.kind, ScopeKind::VFor | ScopeKind::VSlot))
        .map(|s| s.id.as_u32())
        .collect();

    // Root closure scopes: VFor/VSlot scopes whose parent is NOT a closure scope
    let root_closure_scope_ids: FxHashSet<u32> = summary
        .scopes
        .iter()
        .filter(|s| {
            matches!(s.kind, ScopeKind::VFor | ScopeKind::VSlot)
                && s.parent().is_none_or(|pid| {
                    // O(1) arena lookup of the parent scope rather than a
                    // linear find per scope (was O(n^2) over the arena).
                    summary
                        .scopes
                        .get_scope(pid)
                        .is_none_or(|p| !matches!(p.kind, ScopeKind::VFor | ScopeKind::VSlot))
                })
        })
        .map(|s| s.id.as_u32())
        .collect();

    let vfor_enclosing_guards: FxHashMap<u32, String> = summary
        .scopes
        .iter()
        .filter(|scope| matches!(scope.kind, ScopeKind::VFor))
        .filter_map(|scope| {
            let scope_id = scope.id.as_u32();
            let ScopeData::VFor(data) = scope.data() else {
                return None;
            };
            ctx.vfor_enclosing_guards
                .get(&scope_id)
                .map(|guard| (scope_id, guard.clone()))
                .or_else(|| {
                    let usages = components_by_scope.get(&scope_id)?;
                    let mut guards = Vec::new();
                    for (_, usage) in usages {
                        guards.push(usage.vif_guard.as_ref()?.as_str());
                    }
                    common_vif_guard_prefix_for_guards_outside_v_for(guards.as_slice(), data)
                        .map(|guard| (scope_id, guard))
                })
        })
        .collect();

    ts.push_str("\n  // Component props value checks (template scope)\n");
    for &(idx, usage) in checkable_usages {
        if closure_scope_ids.contains(&usage.scope_id.as_u32()) {
            continue; // Will be emitted inside v-for/v-slot scope
        }
        if is_empty_props_usage(usage) {
            continue;
        }
        profile!(
            "canon.virtual_ts.component_prop_checks",
            generate_component_prop_checks(
                ts,
                mappings,
                usage,
                idx,
                ctx.template_prop_names,
                ctx.source_context(),
                "  "
            )
        );
    }

    generate_empty_root_checks(ts, mappings, ctx, checkable_usages, &closure_scope_ids);

    for scope in summary.scopes.iter() {
        if !matches!(scope.kind, ScopeKind::VFor | ScopeKind::VSlot) {
            continue;
        }
        // Only process root closure scopes; nested ones are handled recursively
        if !root_closure_scope_ids.contains(&scope.id.as_u32()) {
            continue;
        }
        let props_ctx = VForPropsContext {
            summary,
            options: ctx.options,
            components_by_scope: &components_by_scope,
            children_map: ctx.children_map,
            vfor_enclosing_guards: &vfor_enclosing_guards,
            template_prop_names: ctx.template_prop_names,
            source_context: ctx.source_context(),
            preserve_event_navigation: ctx.preserve_event_navigation,
        };
        profile!(
            "canon.virtual_ts.closure_component_props",
            generate_closure_component_props_recursive(ts, mappings, &props_ctx, scope, "  ")
        );
    }
}

pub(super) fn collect_checkable_usages<'a>(
    ctx: &ComponentPropsContext<'a>,
) -> Vec<(usize, &'a ComponentUsage)> {
    let external_template_bindings: FxHashSet<&str> = ctx
        .options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    ctx.summary
        .component_usages
        .iter()
        .enumerate()
        .filter(|(_, usage)| {
            component_usage_has_checkable_binding(
                ctx.summary,
                usage,
                &external_template_bindings,
                ctx.check_unresolved_global_components,
                ctx.legacy_vue2,
            )
        })
        .collect()
}

pub(super) fn component_usage_has_checkable_binding(
    summary: &Croquis,
    usage: &ComponentUsage,
    external_template_bindings: &FxHashSet<&str>,
    check_unresolved_global_components: GlobalComponentCheck,
    legacy_vue2: bool,
) -> bool {
    let name = usage.name.as_str();
    summary.bindings.bindings.contains_key(name)
        || (!legacy_vue2
            && (external_template_bindings.contains(name)
                || check_unresolved_global_components.allows(name)))
}

fn generate_closure_component_props_recursive(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &VForPropsContext<'_>,
    scope: &Scope,
    indent: &str,
) {
    let scope_id = scope.id.as_u32();
    let inner_indent = vize_carton::cstr!("{indent}  ");
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

            // Mark v-for variables as used to avoid TS6133
            for value in &data.value_bindings {
                append!(*ts, "{vfor_inner_indent}void {value};\n");
            }
            if let Some(ref key) = data.key_alias {
                append!(*ts, "{vfor_inner_indent}void {key};\n");
            }
            if let Some(ref index) = data.index_alias {
                append!(*ts, "{vfor_inner_indent}void {index};\n");
            }

            // Emit component prop checks for this scope
            generate_scope_checks(ts, mappings, ctx, scope_id, &vfor_inner_indent);

            // Recursively handle child closure scopes (v-for and v-slot)
            recurse_child_closure_scopes(ts, mappings, ctx, scope_id, &vfor_inner_indent);

            ts.push_str(&loop_indent);
            ts.push_str("});\n");
            if enclosing_guard.is_some() {
                append!(*ts, "{indent}}}\n");
            }
        }
        ScopeData::VSlot(data) => {
            let props_pattern = data.props_pattern.as_deref().unwrap_or("slotProps");
            let safe_slot_name = to_safe_identifier_fragment(data.name.as_str());
            append!(
                *ts,
                "\n{indent}// Component props in v-slot scope: #{}\n",
                data.name
            );
            let props_type = slot_props_type(
                ctx.summary,
                ctx.options,
                data.component.as_deref(),
                data.name.as_str(),
                ctx.summary.scopes.is_v_slot_name_static(scope.id),
            );
            emit_slot_function_open(
                ts,
                indent,
                cstr!("_slot_props_{safe_slot_name}_{}", scope.id.as_u32()).as_str(),
                props_pattern,
                &props_type,
            );
            // Mark slot prop variables as used
            if data.prop_names.is_empty() {
                append!(*ts, "{inner_indent}void {props_pattern};\n");
            } else {
                for prop_name in data.prop_names.iter() {
                    append!(*ts, "{inner_indent}void {prop_name};\n");
                }
            }
            // Emit component prop checks for this scope
            generate_scope_checks(ts, mappings, ctx, scope_id, &inner_indent);

            // Recursively handle child closure scopes (v-for and v-slot)
            recurse_child_closure_scopes(ts, mappings, ctx, scope_id, &inner_indent);

            ts.push_str(indent);
            ts.push_str("};\n");
        }
        _ => {}
    }
}

/// Recurse into a scope's direct v-for/v-slot child scopes, emitting their
/// component prop checks at the given indent.
fn recurse_child_closure_scopes(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &VForPropsContext<'_>,
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
