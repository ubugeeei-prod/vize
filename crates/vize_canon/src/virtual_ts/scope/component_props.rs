//! Scope-aware component props type checks, including recursion into nested
//! v-for/v-slot closure scopes.

mod closure_scopes;
mod duplicate_listeners;
mod dynamic_reference;
use duplicate_listeners::append_duplicate_listener_checks;
pub(super) use dynamic_reference::named_dynamic_reference;

pub(super) use closure_scopes::generate_closure_component_props_recursive;
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
    usage_needs_per_prop_aliases,
};
use super::component_prop_navigation;
use super::component_slots::{
    ComponentSlotCheckMeta, append_component_slot_check_helpers, generate_component_slot_checks,
};
use super::context::{ComponentBindingCheck, ComponentPropsContext, VForPropsContext};
use super::empty_component_props::{generate_empty_root_checks, is_empty_props_usage};
use super::root_owner_packets::RootOwnerPackets;
use super::vif_guard::common_vif_guard_prefix_for_guards_outside_v_for;

/// Generate component props type checks (scope-aware).
/// Type declarations are at template level, value checks are in their scope.
pub(super) fn generate_component_props(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    diagnostic_mappings: &mut Vec<VizeMapping>,
    semantic_links: &mut Vec<VizeSemanticLink>,
    ctx: &ComponentPropsContext<'_, '_>,
    checkable_usages: &[(usize, &ComponentUsage)],
    root_packets: &mut RootOwnerPackets,
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
    append_prop_check_helpers(
        ts,
        checkable_usages,
        ctx.check_unknown_props,
        ctx.check_unknown_fallthrough_props,
        ctx.strict_component_attrs,
    );
    append_component_slot_check_helpers(ts, ctx.experimental_strict_slot_children);

    let shared_contracts = shared_component_contracts(ctx, checkable_usages);

    for &(idx, usage) in checkable_usages {
        if !root_packets.contains(idx) {
            generate_usage_declarations(ts, mappings, ctx, &shared_contracts, idx, usage);
        }
    }
    let legacy_usages;
    let navigation_usages = if root_packets.is_empty() {
        checkable_usages
    } else {
        legacy_usages = checkable_usages
            .iter()
            .copied()
            .filter(|(idx, _)| !root_packets.contains(*idx))
            .collect::<Vec<_>>();
        &legacy_usages
    };

    component_prop_navigation::emit_references(
        ts,
        mappings,
        semantic_links,
        ctx,
        navigation_usages,
    );

    // Collect all scopes whose component checks need a replayed environment.
    let closure_scope_ids: FxHashSet<u32> = summary
        .scopes
        .iter()
        .filter(|s| {
            matches!(
                s.kind,
                ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
            )
        })
        .map(|s| s.id.as_u32())
        .collect();

    // Nested closure scopes are replayed recursively from their root.
    let root_closure_scope_ids: FxHashSet<u32> = summary
        .scopes
        .iter()
        .filter(|s| {
            matches!(
                s.kind,
                ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
            ) && s.parent().is_none_or(|pid| {
                // O(1) arena lookup of the parent scope rather than a
                // linear find per scope (was O(n^2) over the arena).
                summary.scopes.get_scope(pid).is_none_or(|p| {
                    !matches!(
                        p.kind,
                        ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
                    )
                })
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
            continue; // Emitted inside the replayed closure scope.
        }
        if is_empty_props_usage(usage) {
            continue;
        }
        if !root_packets.contains(idx) {
            generate_root_usage_checks(ts, mappings, ctx, idx, usage);
        }
    }

    generate_empty_root_checks(ts, mappings, ctx, checkable_usages, &closure_scope_ids);

    for scope in summary.scopes.iter() {
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
            template_binding_access: ctx.template_binding_access,
            syntactic_type_only_imported_names: ctx.syntactic_type_only_imported_names,
            source_context: ctx.source_context(),
            preserve_event_navigation: ctx.preserve_event_navigation,
            check_unknown_events: ctx.check_unknown_events,
            strict_v_model: ctx.strict_v_model,
            experimental_strict_slot_children: ctx.experimental_strict_slot_children,
            explicit_generics: ctx.explicit_generics,
        };
        profile!(
            "canon.virtual_ts.closure_component_props",
            generate_closure_component_props_recursive(ts, mappings, &props_ctx, scope, "  ")
        );
    }
    for &(idx, usage) in checkable_usages {
        if let Some(mut packet) = root_packets.remove(idx) {
            generate_usage_declarations(
                &mut packet.text,
                &mut packet.mappings,
                ctx,
                &shared_contracts,
                idx,
                usage,
            );
            component_prop_navigation::emit_usage_references(
                &mut packet.text,
                &mut packet.mappings,
                &mut packet.semantic_links,
                ctx,
                idx,
                usage,
            );
            generate_root_usage_checks(&mut packet.text, &mut packet.mappings, ctx, idx, usage);
            packet.append_to(ts, mappings, diagnostic_mappings, semantic_links);
        }
    }
}

fn generate_usage_declarations(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_, '_>,
    shared_contracts: &FxHashMap<usize, SharedComponentContract>,
    idx: usize,
    usage: &ComponentUsage,
) {
    let summary = ctx.summary;
    let component_ref = ctx.explicit_generics.usage_reference(
        usage.start,
        named_dynamic_reference(ctx, usage).unwrap_or_else(|| {
            component_binding_reference(
                summary,
                ctx.options,
                ctx.syntactic_type_only_imported_names,
                usage.name.as_str(),
            )
        }),
    );
    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());

    let src_start = (ctx.template_offset + usage.start) as usize;
    let src_end = (ctx.template_offset + usage.end) as usize;
    append!(*ts, "  // @vize-map: component -> {src_start}:{src_end}\n",);
    // Prefer the modern static raw-props identity while retaining the
    // instance marker for declarations emitted before #4034.
    append!(*ts, "  type __{component_type_name}_Props_{idx} = ",);
    let shared = shared_contracts.get(&idx);
    if let Some(shared) = shared.filter(|shared| shared.canonical_idx != idx) {
        append!(
            *ts,
            "__{component_type_name}_Props_{};\n",
            shared.canonical_idx,
        );
    } else {
        append!(
            *ts,
            "typeof {component_ref} extends {{ __vizeCheck: infer __F }} ? (__VizeIsAny<__F> extends true ? __VizeComponentRawProps<typeof {component_ref}> : Record<string, unknown>) : __VizeComponentRawProps<typeof {component_ref}>;\n",
        );
    }

    // Generic functional prop-checker for this component (#775).
    append_prop_checker_alias(
        ts,
        usage,
        component_type_name.as_str(),
        component_ref.as_str(),
        idx,
        ctx.relaxed_required_usage_starts.contains(&usage.start),
        (
            shared
                .filter(|shared| shared.canonical_idx != idx)
                .map(|shared| shared.canonical_idx),
            shared.is_some_and(|shared| shared.needs_value_props),
        ),
    );

    append_per_prop_aliases(ts, usage, component_type_name.as_str(), idx);
    append_duplicate_listener_checks(ts, mappings, ctx, usage);
}

#[derive(Clone, Copy)]
struct SharedComponentContract {
    canonical_idx: usize,
    needs_value_props: bool,
}

fn shared_component_contracts(
    ctx: &ComponentPropsContext<'_, '_>,
    usages: &[(usize, &ComponentUsage)],
) -> FxHashMap<usize, SharedComponentContract> {
    let mut groups: FxHashMap<(String, String, bool), SharedComponentContract> =
        FxHashMap::default();
    for &(idx, usage) in usages {
        let component_ref = ctx.explicit_generics.usage_reference(
            usage.start,
            named_dynamic_reference(ctx, usage).unwrap_or_else(|| {
                component_binding_reference(
                    ctx.summary,
                    ctx.options,
                    ctx.syntactic_type_only_imported_names,
                    usage.name.as_str(),
                )
            }),
        );
        let key = (
            String::from(to_safe_identifier_fragment(usage.name.as_str()).as_str()),
            component_ref,
            ctx.relaxed_required_usage_starts.contains(&usage.start),
        );
        groups
            .entry(key)
            .and_modify(|shared| {
                shared.needs_value_props |= usage_needs_per_prop_aliases(usage);
            })
            .or_insert(SharedComponentContract {
                canonical_idx: idx,
                needs_value_props: usage_needs_per_prop_aliases(usage),
            });
    }

    let mut contracts = FxHashMap::default();
    for &(idx, usage) in usages {
        let component_ref = ctx.explicit_generics.usage_reference(
            usage.start,
            named_dynamic_reference(ctx, usage).unwrap_or_else(|| {
                component_binding_reference(
                    ctx.summary,
                    ctx.options,
                    ctx.syntactic_type_only_imported_names,
                    usage.name.as_str(),
                )
            }),
        );
        let key = (
            String::from(to_safe_identifier_fragment(usage.name.as_str()).as_str()),
            component_ref,
            ctx.relaxed_required_usage_starts.contains(&usage.start),
        );
        if let Some(shared) = groups.get(&key) {
            contracts.insert(idx, *shared);
        }
    }
    contracts
}

fn generate_root_usage_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_, '_>,
    idx: usize,
    usage: &ComponentUsage,
) {
    let summary = ctx.summary;
    let slot_check_meta = ComponentSlotCheckMeta {
        summary,
        options: ctx.options,
        syntactic_type_only_imported_names: ctx.syntactic_type_only_imported_names,
        template_ast: ctx.template_ast,
        experimental_strict_slot_children: ctx.experimental_strict_slot_children,
    };
    let component_ref = ctx.explicit_generics.usage_reference(
        usage.start,
        component_binding_reference(
            summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        ),
    );
    profile!("canon.virtual_ts.component_prop_checks", {
        let mut check_context = ComponentPropCheckContext::new(
            ts,
            mappings,
            ctx.template_binding_access,
            ctx.source_context(),
            "  ",
        )
        .strict_v_model(ctx.strict_v_model);
        generate_component_prop_checks(&mut check_context, usage, idx, component_ref.as_str())
    });
    profile!("canon.virtual_ts.component_slot_checks", {
        let mut check_context = ComponentPropCheckContext::new(
            ts,
            mappings,
            ctx.template_binding_access,
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

pub(super) fn collect_checkable_usages(
    ctx: &ComponentPropsContext<'_, '_>,
) -> Vec<(usize, ComponentUsage)> {
    let external_template_bindings: FxHashSet<&str> = ctx
        .options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    vize_croquis::facts::component_usage_list(ctx.summary)
        .into_iter()
        .enumerate()
        .filter(|(_, usage)| {
            component_usage_has_checkable_binding(
                ctx.summary,
                usage,
                &external_template_bindings,
                ctx.component_binding_check,
                ctx.legacy_vue2,
            ) || named_dynamic_reference(ctx, usage).is_some()
        })
        .collect()
}

pub(super) fn component_usage_has_checkable_binding(
    summary: &Croquis,
    usage: &ComponentUsage,
    external_template_bindings: &FxHashSet<&str>,
    component_binding_check: ComponentBindingCheck<'_>,
    legacy_vue2: bool,
) -> bool {
    let name = usage.name.as_str();
    super::super::script_facts::contains_binding(summary, name)
        || (!legacy_vue2
            && (component_name_matches_external_template_binding(name, external_template_bindings)
                || component_binding_check.allows(name)
                || vize_croquis::drawer::is_dynamic_component_alias(name)))
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
