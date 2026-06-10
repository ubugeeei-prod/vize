//! Top-level orchestration of scope-closure generation and the recursive
//! v-for/v-slot/event-handler scope-node walker.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;

use vize_croquis::{Croquis, Scope, ScopeData, ScopeId, ScopeKind, naming::to_pascal_case};

use crate::virtual_ts::expressions::generate_expressions;
use crate::virtual_ts::helpers::{
    get_dom_event_type, to_safe_identifier, to_safe_identifier_fragment,
};
use crate::virtual_ts::types::VizeMapping;

use super::component_props::generate_component_props;
use super::context::{
    ComponentPropsContext, EventHandlerExprContext, ScopeGenContext, ScopeGenerationOptions,
};
use super::emit::{
    append_v_for_comment, emit_slot_function_open, emit_v_for_loop_open, slot_props_type,
};
use super::event_handler::generate_event_handler_expressions;
use super::globals::{generate_instance_global_refs, generate_undefined_refs};

/// Generate scope closures from Croquis scope chain.
/// Uses recursive tree-based generation so nested v-for/v-slot scopes
/// are properly contained within their parent closures.
pub(crate) fn generate_scope_closures(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    options: ScopeGenerationOptions<'_>,
) {
    let check_options = options.check_options;
    let virtual_ts_options = options.virtual_ts_options;

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

    // Determine which scopes are nested inside a closure scope (VFor/VSlot).
    // These will be generated recursively inside their parent, not at top level.
    let nested_scope_ids: FxHashSet<ScopeId> =
        profile!("canon.virtual_ts.collect_nested_scope_ids", {
            summary
                .scopes
                .iter()
                .filter(|scope| {
                    scope.parent().is_some_and(|pid| {
                        // Scope ids are arena indices, so resolve the parent
                        // with the O(1) indexed lookup instead of rescanning
                        // every scope (was O(n^2) over the scope arena).
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
            generate_instance_global_refs(
                ts,
                mappings,
                summary,
                template_offset,
                virtual_ts_options
            )
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
                    template_offset,
                    "  ",
                );
            }
            continue;
        }

        let ctx = ScopeGenContext {
            summary,
            expressions_by_scope: &expressions_by_scope,
            children_map: &children_map,
            template_prop_names,
            template_offset,
            check_options,
            template_syntax_quirks: options.template_syntax_quirks,
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

    // Generate component props type checks (scope-aware)
    if check_options.check_props {
        profile!(
            "canon.virtual_ts.component_props",
            generate_component_props(
                ts,
                mappings,
                &ComponentPropsContext {
                    summary,
                    children_map: &children_map,
                    template_prop_names,
                    template_offset,
                    options: virtual_ts_options,
                    check_unresolved_global_components: options.check_unresolved_global_components,
                },
            )
        );
    }
}

/// Recursively generate a scope node (VFor/VSlot/EventHandler) and its nested children.
fn generate_scope_node(
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
            append_v_for_comment(
                ts,
                indent,
                "v-for scope",
                data.value_alias.as_str(),
                data.source.as_str(),
            );

            emit_v_for_loop_open(
                ts,
                indent,
                data.value_alias.as_str(),
                data.key_alias.as_deref(),
                data.index_alias.as_deref(),
                data.source.as_str(),
            );

            // Mark v-for variables as used to avoid TS6133
            for value in &data.value_bindings {
                append!(*ts, "{inner_indent}void {value};\n");
            }
            if let Some(ref key) = data.key_alias {
                append!(*ts, "{inner_indent}void {key};\n");
            }
            if let Some(ref index) = data.index_alias {
                append!(*ts, "{inner_indent}void {index};\n");
            }

            // Generate expressions in this scope
            if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id)
                && ctx.check_options.check_template_bindings
            {
                generate_expressions(
                    ts,
                    mappings,
                    exprs,
                    ctx.template_prop_names,
                    ctx.template_offset,
                    &inner_indent,
                );
            }

            // Recursively generate child scopes inside this closure
            profile!(
                "canon.virtual_ts.child_scopes",
                generate_child_scopes(ts, mappings, ctx, scope_id, &inner_indent)
            );

            ts.push_str(indent);
            ts.push_str("});\n");
        }
        ScopeData::VSlot(data) => {
            append!(*ts, "\n{indent}// v-slot scope: #{}\n", data.name);

            let props_pattern = data.props_pattern.as_deref().unwrap_or("slotProps");
            let safe_slot_name = to_safe_identifier_fragment(data.name.as_str());
            let props_type = slot_props_type(
                data.component.as_deref(),
                data.name.as_str(),
                ctx.summary.scopes.is_v_slot_name_static(scope.id),
            );
            emit_slot_function_open(
                ts,
                indent,
                cstr!("_slot_{safe_slot_name}").as_str(),
                props_pattern,
                &props_type,
            );
            // Mark slot prop variables as used
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
                    ctx.template_offset,
                    &inner_indent,
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
        ScopeData::EventHandler(data) if ctx.check_options.check_emits => {
            append!(*ts, "\n{indent}// @{} handler\n", data.event_name);

            let safe_event_name = to_safe_identifier(data.event_name.as_str());

            if let Some(ref component_name) = data.target_component {
                let component_ref = to_safe_identifier(component_name.as_str());
                let component_type_name = to_safe_identifier_fragment(component_name.as_str());
                let pascal_event = to_pascal_case(data.event_name.as_str());
                let on_handler = cstr!("on{pascal_event}");

                let prop_key = if on_handler.contains(':') {
                    cstr!("\"{}\"", on_handler.as_str())
                } else {
                    on_handler
                };

                // Type alias (block-scoped in TypeScript)
                // Include scope_id to deduplicate when same component+event appears multiple times
                append!(
                    *ts,
                    "{indent}type __{component_type_name}_{scope_id}_{safe_event_name}_prop_args = typeof {component_ref} extends {{ new (): {{ $props: infer __P }} }}\n",
                );
                append!(
                    *ts,
                    "{indent}  ? __P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[]\n",
                );
                append!(
                    *ts,
                    "{indent}  : typeof {component_ref} extends (props: infer __P) => any\n",
                );
                append!(
                    *ts,
                    "{indent}    ? __P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[]\n",
                );
                append!(*ts, "{indent}    : unknown[];\n");
                append!(
                    *ts,
                    "{indent}type __{component_type_name}_{scope_id}_{safe_event_name}_emit_args = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }}\n",
                );
                append!(
                    *ts,
                    "{indent}  ? __EP extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[]\n",
                );
                append!(*ts, "{indent}  : unknown[];\n");
                append!(
                    *ts,
                    "{indent}type __{component_type_name}_{scope_id}_{safe_event_name}_args = unknown[] extends __{component_type_name}_{scope_id}_{safe_event_name}_prop_args ? __{component_type_name}_{scope_id}_{safe_event_name}_emit_args : __{component_type_name}_{scope_id}_{safe_event_name}_prop_args;\n",
                );
                if ctx.template_syntax_quirks {
                    let fallback_event = get_dom_event_type(data.event_name.as_str());
                    append!(
                        *ts,
                        "{indent}type __{component_type_name}_{scope_id}_{safe_event_name}_event = __{component_type_name}_{scope_id}_{safe_event_name}_args extends [] ? any : unknown[] extends __{component_type_name}_{scope_id}_{safe_event_name}_args ? {fallback_event} : __{component_type_name}_{scope_id}_{safe_event_name}_args[0];\n",
                    );
                } else {
                    append!(
                        *ts,
                        "{indent}type __{component_type_name}_{scope_id}_{safe_event_name}_event = __{component_type_name}_{scope_id}_{safe_event_name}_args extends [] ? any : __{component_type_name}_{scope_id}_{safe_event_name}_args[0];\n",
                    );
                }

                let event_type =
                    cstr!("__{component_type_name}_{scope_id}_{safe_event_name}_event");
                append!(*ts, "{indent}(($event: {event_type}) => {{\n");

                profile!(
                    "canon.virtual_ts.event_handler_expressions",
                    generate_event_handler_expressions(
                        ts,
                        mappings,
                        scope_id,
                        &EventHandlerExprContext {
                            expressions_by_scope: ctx.expressions_by_scope,
                            data,
                            event_type: event_type.as_str(),
                            template_prop_names: ctx.template_prop_names,
                            template_offset: ctx.template_offset,
                            indent: &inner_indent,
                        },
                    )
                );

                append!(*ts, "{indent}}})({{}} as {event_type});\n");
            } else {
                let event_type = get_dom_event_type(data.event_name.as_str());
                append!(*ts, "{indent}(($event: {event_type}) => {{\n");

                profile!(
                    "canon.virtual_ts.event_handler_expressions",
                    generate_event_handler_expressions(
                        ts,
                        mappings,
                        scope_id,
                        &EventHandlerExprContext {
                            expressions_by_scope: ctx.expressions_by_scope,
                            data,
                            event_type,
                            template_prop_names: ctx.template_prop_names,
                            template_offset: ctx.template_offset,
                            indent: &inner_indent,
                        },
                    )
                );

                append!(*ts, "{indent}}})({{}} as {event_type});\n");
            }
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
                    ctx.template_offset,
                    indent,
                );
            }
        }
    }
}

/// Recursively generate child scopes that are VFor/VSlot/EventHandler.
fn generate_child_scopes(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ScopeGenContext<'_>,
    parent_scope_id: u32,
    indent: &str,
) {
    if let Some(child_ids) = ctx.children_map.get(&parent_scope_id) {
        for &child_id in child_ids {
            if let Some(child_scope) = ctx.summary.scopes.get_scope(child_id)
                && matches!(
                    child_scope.kind,
                    ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::EventHandler
                )
            {
                profile!(
                    "canon.virtual_ts.scope_node",
                    generate_scope_node(ts, mappings, ctx, child_scope, indent)
                );
            }
        }
    }
}
