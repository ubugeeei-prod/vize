//! Scope-aware component props type checks, including recursion into nested
//! v-for/v-slot closure scopes.

mod closure_scopes;

use closure_scopes::generate_closure_component_props_recursive;
pub(super) use closure_scopes::recurse_child_closure_scopes;
use vize_carton::{FxHashMap, FxHashSet, String, append, camelize, capitalize, profile};
use vize_croquis::{Croquis, ScopeData, ScopeKind, analysis::ComponentUsage};

use crate::virtual_ts::VizeSemanticLink;
use crate::virtual_ts::component_reference::component_binding_reference;
use crate::virtual_ts::expressions::{ComponentPropCheckContext, generate_component_prop_checks};
use crate::virtual_ts::helpers::to_safe_identifier_fragment;
use crate::virtual_ts::types::VizeMapping;

use super::component_prop_checker::{
    append_per_prop_aliases, append_prop_check_helpers, append_prop_checker_alias,
};
use super::component_prop_navigation;
use super::component_slots::{
    ComponentSlotCheckMeta, append_component_slot_check_helpers, generate_component_slot_checks,
};
use super::context::{ComponentPropsContext, GlobalComponentCheck, VForPropsContext};
use super::empty_component_props::{generate_empty_root_checks, is_empty_props_usage};
use super::vif_guard::common_vif_guard_prefix_for_guards_outside_v_for;

/// Generate component props type checks (scope-aware).
/// Type declarations are at template level, value checks are in their scope.
pub(super) fn generate_component_props(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    ctx: &ComponentPropsContext<'_, '_>,
    checkable_usages: &[(usize, &ComponentUsage)],
) {
    let summary = ctx.summary;
    if checkable_usages.is_empty() {
        return;
    }
    let slot_check_meta = ComponentSlotCheckMeta {
        summary,
        options: ctx.options,
        syntactic_type_only_imported_names: ctx.syntactic_type_only_imported_names,
        template_ast: ctx.template_ast,
        experimental_strict_slot_children: ctx.experimental_strict_slot_children,
    };

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
    append_prop_check_helpers(ts, checkable_usages, ctx.check_unknown_props);
    append_component_slot_check_helpers(ts, ctx.experimental_strict_slot_children);

    for &(idx, usage) in checkable_usages {
        let component_ref = component_binding_reference(
            summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        );
        let component_type_name = to_safe_identifier_fragment(usage.name.as_str());

        let src_start = (ctx.template_offset + usage.start) as usize;
        let src_end = (ctx.template_offset + usage.end) as usize;
        append!(*ts, "  // @vize-map: component -> {src_start}:{src_end}\n",);
        // Prefer the modern static raw-props identity while retaining the
        // instance marker for declarations emitted before #4034.
        append!(
            *ts,
            "  type __{component_type_name}_Props_{idx} = typeof {component_ref} extends {{ __vizeCheck: infer __F }} ? (__VizeIsAny<__F> extends true ? __VizeComponentRawProps<typeof {component_ref}> : Record<string, unknown>) : __VizeComponentRawProps<typeof {component_ref}>;\n",
        );

        // Generic functional prop-checker for this component (#775).
        append_prop_checker_alias(
            ts,
            usage,
            component_type_name.as_str(),
            component_ref.as_str(),
            idx,
        );

        append_per_prop_aliases(ts, usage, component_type_name.as_str(), idx);
    }

    component_prop_navigation::emit_references(ts, mappings, semantic_links, ctx, checkable_usages);

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
        let component_ref = component_binding_reference(
            summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        );
        profile!("canon.virtual_ts.component_prop_checks", {
            let mut check_context = ComponentPropCheckContext::new(
                ts,
                mappings,
                ctx.template_prop_names,
                ctx.source_context(),
                "  ",
            );
            generate_component_prop_checks(&mut check_context, usage, idx, component_ref.as_str())
        });
        profile!("canon.virtual_ts.component_slot_checks", {
            let mut check_context = ComponentPropCheckContext::new(
                ts,
                mappings,
                ctx.template_prop_names,
                ctx.source_context(),
                "  ",
            );
            generate_component_slot_checks(
                &mut check_context,
                usage,
                idx,
                component_ref.as_str(),
                slot_check_meta,
            )
        });
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
            template_ast: ctx.template_ast,
            components_by_scope: &components_by_scope,
            children_map: ctx.children_map,
            vfor_enclosing_guards: &vfor_enclosing_guards,
            template_prop_names: ctx.template_prop_names,
            syntactic_type_only_imported_names: ctx.syntactic_type_only_imported_names,
            source_context: ctx.source_context(),
            preserve_event_navigation: ctx.preserve_event_navigation,
            experimental_strict_slot_children: ctx.experimental_strict_slot_children,
        };
        profile!(
            "canon.virtual_ts.closure_component_props",
            generate_closure_component_props_recursive(ts, mappings, &props_ctx, scope, "  ")
        );
    }
}

pub(super) fn collect_checkable_usages<'a>(
    ctx: &ComponentPropsContext<'a, '_>,
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
            && (component_name_matches_external_template_binding(name, external_template_bindings)
                || check_unresolved_global_components.allows(name)))
}

fn component_name_matches_external_template_binding(
    name: &str,
    external_template_bindings: &FxHashSet<&str>,
) -> bool {
    let camel_name = camelize(name);
    let pascal_name = capitalize(camel_name.as_str());
    [name, camel_name.as_str(), pascal_name.as_str()]
        .iter()
        .any(|candidate| external_template_bindings.contains(candidate))
}
