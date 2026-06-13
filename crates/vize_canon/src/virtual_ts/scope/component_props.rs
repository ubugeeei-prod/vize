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
use crate::virtual_ts::helpers::{to_camel_case, to_safe_identifier, to_safe_identifier_fragment};
use crate::virtual_ts::types::VizeMapping;

use super::context::{ComponentPropsContext, VForPropsContext};
use super::emit::{
    append_v_for_comment, emit_slot_function_open, emit_v_for_loop_open, slot_props_type,
};

/// Generate component props type checks (scope-aware).
/// Type declarations are at template level, value checks are in their scope.
pub(super) fn generate_component_props(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_>,
) {
    let summary = ctx.summary;
    if summary.component_usages.is_empty() {
        return;
    }

    let external_template_bindings: FxHashSet<&str> = ctx
        .options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    let checkable_usages: Vec<(usize, &ComponentUsage)> = summary
        .component_usages
        .iter()
        .enumerate()
        .filter(|(_, usage)| {
            component_usage_has_checkable_binding(
                summary,
                usage,
                &external_template_bindings,
                ctx.check_unresolved_global_components,
            )
        })
        .collect();
    if checkable_usages.is_empty() {
        return;
    }

    // Group component usages by scope_id
    let mut components_by_scope: FxHashMap<u32, Vec<(usize, &ComponentUsage)>> =
        FxHashMap::default();
    for &(idx, usage) in &checkable_usages {
        components_by_scope
            .entry(usage.scope_id.as_u32())
            .or_default()
            .push((idx, usage));
    }

    // Emit type declarations only for components with dynamic props
    // (TypeScript type aliases cannot be inside function bodies)
    ts.push_str("\n  // Component props type declarations\n");

    // Helper types for the generic functional prop-check path (#775). A
    // `<script setup generic="T">` child exposes `__vizeCheck<T>(props)` on its
    // default export; `__VizePropChecker<C>` extracts that generic signature so
    // the parent can call it and let TS infer `T` from the passed props. When
    // the child is non-generic (plain construct signature), a built-in / library
    // component, or `any`, it falls back to `(props: any) => void`, a no-op that
    // never reports, so only generic components take the new path and the
    // well-tested `typeof Comp extends { $props }` extraction below is preserved.
    let any_dynamic_props = checkable_usages.iter().any(|(_, usage)| {
        usage.props.iter().any(|p| {
            p.name.as_str() != "key"
                && p.name.as_str() != "ref"
                && p.value.is_some()
                && p.is_dynamic
        })
    });
    if any_dynamic_props {
        ts.push_str("  type __VizeIsAny<T> = 0 extends (1 & T) ? true : false;\n");
        ts.push_str(
            "  type __VizePropChecker<C> = __VizeIsAny<C> extends true ? (props: any) => void : C extends { __vizeCheck: infer __F } ? (__F extends (...args: any[]) => any ? __F : (props: any) => void) : (props: any) => void;\n",
        );
        ts.push_str(
            "  type __VizePropValue<P, K extends PropertyKey> = K extends keyof P ? P[K] : unknown;\n",
        );
    }

    for &(idx, usage) in &checkable_usages {
        let component_ref = to_safe_identifier(usage.name.as_str());
        let component_type_name = to_safe_identifier_fragment(usage.name.as_str());

        // Only emit type when there are dynamic props to check
        let has_dynamic_props = usage.props.iter().any(|p| {
            p.name.as_str() != "key"
                && p.name.as_str() != "ref"
                && p.value.is_some()
                && p.is_dynamic
        });
        if !has_dynamic_props {
            continue;
        }

        let src_start = (ctx.template_offset + usage.start) as usize;
        let src_end = (ctx.template_offset + usage.end) as usize;

        append!(*ts, "  // @vize-map: component -> {src_start}:{src_end}\n",);
        append!(
            *ts,
            "  type __{component_type_name}_Props_{idx} = typeof {component_ref} extends {{ new (): {{ $props: infer __P }} }} ? __P : (typeof {component_ref} extends (props: infer __P) => any ? __P : {{}});\n",
        );

        for prop in &usage.props {
            if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
                continue;
            }
            if prop.value.is_some() && prop.is_dynamic {
                let camel_prop_name = to_camel_case(prop.name.as_str());
                let safe_prop_name = to_safe_identifier_fragment(prop.name.as_str());
                append!(
                    *ts,
                    "  type __{component_type_name}_{idx}_prop_{safe_prop_name} = __VizePropValue<__{component_type_name}_Props_{idx}, '{camel_prop_name}'>;\n",
                );
            }
        }

        // Generic functional prop-checker for this component (#775). Resolves to
        // the child's `__vizeCheck` when generic, else a `(props: any)` no-op.
        append!(
            *ts,
            "  type __{component_type_name}_Check_{idx} = __VizePropChecker<typeof {component_ref}>;\n",
        );
    }

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

    ts.push_str("\n  // Component props value checks (template scope)\n");
    for &(idx, usage) in &checkable_usages {
        if closure_scope_ids.contains(&usage.scope_id.as_u32()) {
            continue; // Will be emitted inside v-for/v-slot scope
        }
        profile!(
            "canon.virtual_ts.component_prop_checks",
            generate_component_prop_checks(
                ts,
                mappings,
                usage,
                idx,
                ctx.template_prop_names,
                ctx.template_offset,
                "  "
            )
        );
    }

    // Emit value checks for components in closure scopes (v-for and v-slot)
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
            components_by_scope: &components_by_scope,
            children_map: ctx.children_map,
            vfor_enclosing_guards: ctx.vfor_enclosing_guards,
            template_prop_names: ctx.template_prop_names,
            template_offset: ctx.template_offset,
        };
        profile!(
            "canon.virtual_ts.closure_component_props",
            generate_closure_component_props_recursive(ts, mappings, &props_ctx, scope, "  ")
        );
    }
}

fn component_usage_has_checkable_binding(
    summary: &Croquis,
    usage: &ComponentUsage,
    external_template_bindings: &FxHashSet<&str>,
    check_unresolved_global_components: bool,
) -> bool {
    let name = usage.name.as_str();
    summary.bindings.bindings.contains_key(name)
        || external_template_bindings.contains(name)
        || (check_unresolved_global_components && !name.is_empty())
}

/// Recursively generate component prop checks inside nested closure scopes (v-for and v-slot).
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
                &loop_indent,
                data.value_alias.as_str(),
                data.key_alias.as_deref(),
                data.index_alias.as_deref(),
                data.source.as_str(),
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
            if let Some(usages) = ctx.components_by_scope.get(&scope_id) {
                for &(idx, usage) in usages {
                    profile!(
                        "canon.virtual_ts.component_prop_checks",
                        generate_component_prop_checks(
                            ts,
                            mappings,
                            usage,
                            idx,
                            ctx.template_prop_names,
                            ctx.template_offset,
                            &vfor_inner_indent,
                        )
                    );
                }
            }

            // Recursively handle child closure scopes (v-for and v-slot)
            if let Some(child_ids) = ctx.children_map.get(&scope_id) {
                for &child_id in child_ids {
                    if let Some(child_scope) = ctx.summary.scopes.get_scope(child_id)
                        && matches!(child_scope.kind, ScopeKind::VFor | ScopeKind::VSlot)
                    {
                        profile!(
                            "canon.virtual_ts.closure_component_props",
                            generate_closure_component_props_recursive(
                                ts,
                                mappings,
                                ctx,
                                child_scope,
                                &vfor_inner_indent,
                            )
                        );
                    }
                }
            }

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
                data.component.as_deref(),
                data.name.as_str(),
                ctx.summary.scopes.is_v_slot_name_static(scope.id),
            );
            emit_slot_function_open(
                ts,
                indent,
                cstr!("_slot_props_{safe_slot_name}").as_str(),
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
            if let Some(usages) = ctx.components_by_scope.get(&scope_id) {
                for &(idx, usage) in usages {
                    profile!(
                        "canon.virtual_ts.component_prop_checks",
                        generate_component_prop_checks(
                            ts,
                            mappings,
                            usage,
                            idx,
                            ctx.template_prop_names,
                            ctx.template_offset,
                            &inner_indent,
                        )
                    );
                }
            }

            // Recursively handle child closure scopes (v-for and v-slot)
            if let Some(child_ids) = ctx.children_map.get(&scope_id) {
                for &child_id in child_ids {
                    if let Some(child_scope) = ctx.summary.scopes.get_scope(child_id)
                        && matches!(child_scope.kind, ScopeKind::VFor | ScopeKind::VSlot)
                    {
                        profile!(
                            "canon.virtual_ts.closure_component_props",
                            generate_closure_component_props_recursive(
                                ts,
                                mappings,
                                ctx,
                                child_scope,
                                &inner_indent,
                            )
                        );
                    }
                }
            }

            ts.push_str(indent);
            ts.push_str("};\n");
        }
        _ => {}
    }
}
