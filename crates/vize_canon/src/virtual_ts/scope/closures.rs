//! Top-level orchestration of scope-closure generation and the recursive
//! v-for/v-slot/event-handler scope-node walker.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;

use vize_croquis::{Croquis, Scope, ScopeData, ScopeId, ScopeKind};

use crate::virtual_ts::expressions::{
    ExpressionListEmitContext, TemplateValueCheckTables, generate_expressions,
    generate_expressions_in_enclosing_guard,
};
use crate::virtual_ts::helpers::to_safe_identifier_fragment;
use crate::virtual_ts::types::VizeMapping;

use super::children::generate_child_scopes;
use super::component_prop_expressions::collect_component_prop_expression_ranges;
use super::component_props::generate_component_props;
use super::context::{ComponentPropsContext, ScopeGenContext, ScopeGenerationOptions};
use super::emit::{
    append_v_for_comment, emit_slot_function_open, emit_v_for_loop_open, slot_props_type,
};
use super::event_scope::generate_event_handler_scope;
use super::globals::{generate_instance_global_refs, generate_undefined_refs};
use super::vif_guard::{callback_vif_guard, common_vif_guard_prefix_outside_v_for_scope};

/// Generate scope closures from Croquis scope chain.
/// Uses recursive tree-based generation so nested v-for/v-slot scopes
/// are properly contained within their parent closures.
pub(crate) fn generate_scope_closures(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    options: ScopeGenerationOptions<'_, '_>,
) {
    let check_options = options.check_options;
    let virtual_ts_options = options.virtual_ts_options;
    let check_tables = TemplateValueCheckTables::collect(summary, &options);
    let checks = check_tables.as_checks();

    // Group expressions by scope_id
    let expressions_by_scope: FxHashMap<u32, Vec<_>> =
        profile!("canon.virtual_ts.group_template_expressions", {
            let mut expressions_by_scope: FxHashMap<u32, Vec<_>> = FxHashMap::default();
            for expr in &summary.template_expressions {
                expressions_by_scope
                    .entry(expr.scope_id.as_u32())
                    .or_default()
                    .push(expr);
            }
            expressions_by_scope
        });
    let skipped_expression_ranges =
        profile!("canon.virtual_ts.component_prop_expression_ranges", {
            collect_component_prop_expression_ranges(summary, virtual_ts_options, &options)
        });

    // Build scope tree: parent_scope_id -> Vec<child ScopeId>
    let children_map: FxHashMap<u32, Vec<ScopeId>> =
        profile!("canon.virtual_ts.build_scope_tree", {
            let mut children_map: FxHashMap<u32, Vec<ScopeId>> = FxHashMap::default();
            for scope in summary.scopes.iter() {
                if let Some(parent_id) = scope.parent() {
                    children_map
                        .entry(parent_id.as_u32())
                        .or_default()
                        .push(scope.id);
                }
            }
            children_map
        });

    let vfor_enclosing_guards: FxHashMap<u32, String> =
        profile!("canon.virtual_ts.vfor_enclosing_guards", {
            if !check_options.check_template_bindings {
                FxHashMap::default()
            } else {
                summary
                    .scopes
                    .iter()
                    .filter(|scope| matches!(scope.kind, ScopeKind::VFor))
                    .filter_map(|scope| {
                        let scope_id = scope.id.as_u32();
                        expressions_by_scope
                            .get(&scope_id)
                            .and_then(|exprs| {
                                common_vif_guard_prefix_outside_v_for_scope(exprs, scope)
                            })
                            .map(|guard| (scope_id, guard))
                    })
                    .collect()
            }
        });

    // Scopes nested inside a VFor/VSlot closure are generated inside their parent.
    let nested_scope_ids: FxHashSet<ScopeId> =
        profile!("canon.virtual_ts.collect_nested_scope_ids", {
            summary
                .scopes
                .iter()
                .filter(|scope| {
                    scope.parent().is_some_and(|pid| {
                        // Resolve the parent via O(1) indexed lookup (was O(n^2)).
                        summary.scopes.get_scope(pid).is_some_and(|parent| {
                            matches!(parent.kind, ScopeKind::VFor | ScopeKind::VSlot)
                        })
                    })
                })
                .map(|scope| scope.id)
                .collect()
        });

    if check_options.check_template_bindings {
        profile!(
            "canon.virtual_ts.instance_global_refs",
            generate_instance_global_refs(ts, mappings, summary, template_offset, &options)
        );
    }

    // Process non-nested scopes at template level
    for scope in summary.scopes.iter() {
        let scope_id = scope.id.as_u32();

        // Skip scopes that are nested inside a closure parent
        if nested_scope_ids.contains(&scope.id) {
            continue;
        }

        // Global scopes: emit expressions directly
        if matches!(
            scope.kind,
            ScopeKind::JsGlobalUniversal
                | ScopeKind::JsGlobalBrowser
                | ScopeKind::JsGlobalNode
                | ScopeKind::VueGlobal
        ) {
            if let Some(exprs) = expressions_by_scope.get(&scope_id)
                && check_options.check_template_bindings
            {
                generate_expressions(
                    ts,
                    mappings,
                    exprs,
                    template_prop_names,
                    &ExpressionListEmitContext::new(
                        &skipped_expression_ranges,
                        template_offset,
                        "  ",
                        checks,
                    ),
                );
            }
            continue;
        }
        let ctx = ScopeGenContext {
            summary,
            virtual_ts_options,
            expressions_by_scope: &expressions_by_scope,
            skipped_expression_ranges: &skipped_expression_ranges,
            children_map: &children_map,
            template_prop_names,
            checks,
            template_source: options.template_ast.map(|root| root.source.as_str()),
            template_offset,
            check_options,
            legacy_vue2: options.legacy_vue2,
        };
        profile!(
            "canon.virtual_ts.scope_node",
            generate_scope_node(ts, mappings, &ctx, scope, "  ")
        );
    }

    // Handle undefined references
    if check_options.check_template_bindings {
        profile!(
            "canon.virtual_ts.undefined_refs",
            generate_undefined_refs(ts, mappings, summary, template_offset)
        );
    }
    if check_options.check_props {
        profile!(
            "canon.virtual_ts.component_props",
            generate_component_props(
                ts,
                mappings,
                &ComponentPropsContext {
                    summary,
                    template_source: options.template_ast.map(|root| root.source.as_str()),
                    children_map: &children_map,
                    vfor_enclosing_guards: &vfor_enclosing_guards,
                    template_prop_names,
                    template_offset,
                    options: virtual_ts_options,
                    check_unresolved_global_components: options.check_unresolved_global_components,
                    legacy_vue2: options.legacy_vue2,
                },
            )
        );
    }
}

/// Recursively generate a scope node (VFor/VSlot/EventHandler) and its nested children.
pub(super) fn generate_scope_node(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ScopeGenContext<'_>,
    scope: &Scope,
    indent: &str,
) {
    let scope_id = scope.id.as_u32();
    let inner_indent = cstr!("{indent}  ");

    match scope.data() {
        ScopeData::VFor(data) => {
            // For a v-for nested in `v-if`, wrap the loop in `if (guard) {}` so
            // TypeScript narrows identifiers in the v-for source (e.g. `elems[key]`
            // narrowed by the parent `v-if`); otherwise it widens (#1511). The guard
            // is the longest common `&&`-separated prefix of this scope's expressions.
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
                append!(*ts, "{indent}if ({guard}) {{\n");
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
                data,
                ctx.template_prop_names,
            );
            // A positive narrowing outside a callback is not retained for captured
            // object properties. Recheck those terms so discriminated unions narrow.
            let callback_guard = enclosing_guard.and_then(callback_vif_guard);
            let callback_indent = if let Some(guard) = callback_guard.as_deref() {
                append!(*ts, "{vfor_inner_indent}if ({guard}) {{\n");
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

            // Generate expressions in this scope
            if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id)
                && ctx.check_options.check_template_bindings
            {
                generate_expressions_in_enclosing_guard(
                    ts,
                    mappings,
                    exprs,
                    ctx.template_prop_names,
                    &ExpressionListEmitContext::new(
                        ctx.skipped_expression_ranges,
                        ctx.template_offset,
                        &callback_indent,
                        ctx.checks,
                    ),
                    enclosing_guard,
                );
            }

            // Recursively generate child scopes inside this closure
            profile!(
                "canon.virtual_ts.child_scopes",
                generate_child_scopes(ts, mappings, ctx, scope_id, &callback_indent)
            );

            if callback_guard.is_some() {
                append!(*ts, "{vfor_inner_indent}}}\n");
            }

            ts.push_str(&loop_indent);
            ts.push_str("});\n");

            if enclosing_guard.is_some() {
                append!(*ts, "{indent}}}\n");
            }
        }
        ScopeData::VSlot(data) => {
            append!(*ts, "\n{indent}// v-slot scope: #{}\n", data.name);
            let props_pattern = data.props_pattern.as_deref().unwrap_or("slotProps");
            let safe_slot_name = to_safe_identifier_fragment(data.name.as_str());
            let props_type = slot_props_type(
                ctx.summary,
                ctx.virtual_ts_options,
                data.component.as_deref(),
                data.name.as_str(),
                ctx.summary.scopes.is_v_slot_name_static(scope.id),
            );
            emit_slot_function_open(
                ts,
                indent,
                cstr!("_slot_{safe_slot_name}_{scope_id}").as_str(),
                props_pattern,
                &props_type,
            );
            if data.prop_names.is_empty() {
                // Simple identifier (no destructuring)
                append!(*ts, "{inner_indent}void {props_pattern};\n");
            } else {
                // Destructured: void each extracted prop name
                for prop_name in data.prop_names.iter() {
                    append!(*ts, "{inner_indent}void {prop_name};\n");
                }
            }

            if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id)
                && ctx.check_options.check_template_bindings
            {
                generate_expressions(
                    ts,
                    mappings,
                    exprs,
                    ctx.template_prop_names,
                    &ExpressionListEmitContext::new(
                        ctx.skipped_expression_ranges,
                        ctx.template_offset,
                        &inner_indent,
                        ctx.checks,
                    ),
                );
            }

            // Recursively generate child scopes inside this closure
            profile!(
                "canon.virtual_ts.child_scopes",
                generate_child_scopes(ts, mappings, ctx, scope_id, &inner_indent)
            );

            ts.push_str(indent);
            ts.push_str("};\n");
        }
        ScopeData::EventHandler(data) if ctx.check_options.check_event_handlers() => {
            generate_event_handler_scope(ts, mappings, ctx, scope, data, indent, &inner_indent);
        }
        _ => {
            if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id)
                && ctx.check_options.check_template_bindings
            {
                generate_expressions(
                    ts,
                    mappings,
                    exprs,
                    ctx.template_prop_names,
                    &ExpressionListEmitContext::new(
                        ctx.skipped_expression_ranges,
                        ctx.template_offset,
                        indent,
                        ctx.checks,
                    ),
                );
            }
        }
    }
}
