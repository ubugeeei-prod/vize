//! Scope-aware component props type checks, including recursion into nested
//! v-for/v-slot closure scopes.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;

use vize_croquis::{
    Croquis, Scope, ScopeData, ScopeKind,
    analysis::{ComponentUsage, PassedProp},
};

use crate::virtual_ts::expressions::generate_component_prop_checks;
use crate::virtual_ts::helpers::{to_camel_case, to_safe_identifier, to_safe_identifier_fragment};
use crate::virtual_ts::types::VizeMapping;

use super::component_prop_checker::append_prop_checker_alias;
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
                ctx.legacy_vue2,
            )
        })
        .collect();
    if checkable_usages.is_empty() {
        return;
    }

    let mut components_by_scope: FxHashMap<u32, Vec<(usize, &ComponentUsage)>> =
        FxHashMap::default();
    for &(idx, usage) in &checkable_usages {
        components_by_scope
            .entry(usage.scope_id.as_u32())
            .or_default()
            .push((idx, usage));
    }

    ts.push_str("\n  // Component props type declarations\n");

    // Generic children expose `__vizeCheck<T>(props)`; fallback contextual
    // typing is limited to inline function props to avoid duplicate errors.
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
            "  type __VizePropChecker<C, P> = __VizeIsAny<C> extends true ? (props: P & Record<string, unknown>) => void : C extends { __vizeCheck: infer __F } ? (__F extends (...args: any[]) => any ? __F : (props: P & Record<string, unknown>) => void) : (props: P & Record<string, unknown>) => void;\n",
        );
        ts.push_str(
            "  type __VizePropValue<P, K extends PropertyKey, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? unknown : __V;\n",
        );
    }

    for &(idx, usage) in &checkable_usages {
        let component_ref = to_safe_identifier(usage.name.as_str());
        let component_type_name = to_safe_identifier_fragment(usage.name.as_str());

        let has_dynamic_props = usage.props.iter().any(|p| {
            p.name.as_str() != "key"
                && p.name.as_str() != "ref"
                && p.value.is_some()
                && p.is_dynamic
        });
        let has_navigable_props = usage.props.iter().any(|p| {
            p.name.as_str() != "key"
                && p.name.as_str() != "ref"
                && prop_navigation_source_range(ctx.template_source, p).is_some()
        });
        if !has_dynamic_props && !has_navigable_props {
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

        if has_dynamic_props {
            // Generic functional prop-checker for this component (#775).
            append_prop_checker_alias(
                ts,
                usage,
                component_type_name.as_str(),
                component_ref.as_str(),
                idx,
            );
        }
    }

    emit_component_navigation_references(ts, mappings, ctx, &checkable_usages);

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

fn emit_component_navigation_references(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_>,
    checkable_usages: &[(usize, &ComponentUsage)],
) {
    ts.push_str("\n  // Component template navigation references\n");
    for &(idx, usage) in checkable_usages {
        let component_ref = to_safe_identifier(usage.name.as_str());
        let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
        let tag_src_start = (ctx.template_offset + usage.start + 1) as usize;
        let tag_src_end = tag_src_start + usage.name.len();

        ts.push_str("  void ");
        let tag_gen_start = ts.len();
        ts.push_str(&component_ref);
        let tag_gen_end = ts.len();
        ts.push_str(";\n");
        mappings.push(VizeMapping {
            gen_range: tag_gen_start..tag_gen_end,
            src_range: tag_src_start..tag_src_end,
            sub_spans: Vec::new(),
        });

        let props_ref = cstr!("__vize_props_nav_{idx}");
        let mut emitted_props_ref = false;
        for prop in &usage.props {
            if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
                continue;
            }
            let Some(source_range) = prop_navigation_source_range(ctx.template_source, prop) else {
                continue;
            };

            if !emitted_props_ref {
                append!(
                    *ts,
                    "  const {props_ref} = undefined as unknown as __{component_type_name}_Props_{idx} & Record<string, unknown>;\n"
                );
                emitted_props_ref = true;
            }

            let camel_prop_name = to_camel_case(prop.name.as_str());
            append!(*ts, "  void {props_ref}");
            let prop_gen_range = if is_ts_identifier(camel_prop_name.as_str()) {
                ts.push('.');
                let prop_gen_start = ts.len();
                ts.push_str(camel_prop_name.as_str());
                prop_gen_start..ts.len()
            } else {
                ts.push('[');
                let range = push_ts_single_quoted_literal(ts, camel_prop_name.as_str());
                ts.push(']');
                range
            };
            ts.push_str(";\n");
            mappings.push(VizeMapping {
                gen_range: prop_gen_range,
                src_range: (ctx.template_offset as usize + source_range.start)
                    ..(ctx.template_offset as usize + source_range.end),
                sub_spans: Vec::new(),
            });
        }
    }
}

fn prop_navigation_source_range(
    template_source: Option<&str>,
    prop: &PassedProp,
) -> Option<std::ops::Range<usize>> {
    let name = prop.name.as_str();
    if name.is_empty() {
        return None;
    }

    let start = prop.start as usize;
    let end = prop.end as usize;
    let source = template_source?;
    let raw = source.get(start..end)?;
    if let Some(relative_start) = raw.find(name) {
        return Some(start + relative_start..start + relative_start + name.len());
    }

    if name == "modelValue"
        && let Some(relative_start) = raw.find("v-model")
    {
        return Some(start + relative_start..start + relative_start + "v-model".len());
    }

    None
}

fn push_ts_single_quoted_literal(ts: &mut String, value: &str) -> std::ops::Range<usize> {
    ts.push('\'');
    let start = ts.len();
    for ch in value.chars() {
        match ch {
            '\\' => ts.push_str("\\\\"),
            '\'' => ts.push_str("\\'"),
            '\n' => ts.push_str("\\n"),
            '\r' => ts.push_str("\\r"),
            '\t' => ts.push_str("\\t"),
            _ => ts.push(ch),
        }
    }
    let end = ts.len();
    ts.push('\'');
    start..end
}

fn is_ts_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

pub(super) fn component_usage_has_checkable_binding(
    summary: &Croquis,
    usage: &ComponentUsage,
    external_template_bindings: &FxHashSet<&str>,
    check_unresolved_global_components: bool,
    legacy_vue2: bool,
) -> bool {
    let name = usage.name.as_str();
    summary.bindings.bindings.contains_key(name)
        || (!legacy_vue2
            && (external_template_bindings.contains(name)
                || (check_unresolved_global_components && !name.is_empty())))
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
