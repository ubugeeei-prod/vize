//! Top-level orchestration of scope-closure generation and the recursive
//! v-for/v-slot/event-handler scope-node walker.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;

use vize_croquis::{Croquis, Scope, ScopeData, ScopeId, ScopeKind};

use crate::virtual_ts::expressions::generate_expressions;
use crate::virtual_ts::helpers::{get_dom_event_type, to_safe_identifier_fragment};
use crate::virtual_ts::types::VizeMapping;

use super::component_events::generate_component_event_types;
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
                            .and_then(|exprs| common_vif_guard_prefix(exprs))
                            .map(|guard| (scope_id, guard))
                    })
                    .collect()
            }
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
                    vfor_enclosing_guards: &vfor_enclosing_guards,
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
            // When the v-for element is nested inside an enclosing `v-if`, wrap
            // the whole `__vForList(source).forEach(...)` loop in `if (guard) {}`
            // so TypeScript narrows identifiers used in the v-for source
            // expression (e.g. `elems[key]` with `key` narrowed by the parent
            // `v-if="key === 'b'"`). Without this the source is evaluated outside
            // the narrowing scope and yields the unnarrowed (wider) type (#1511).
            //
            // The v-for element's own bindings (`:key`, etc.) and interpolations
            // are recorded as expressions in this scope carrying that enclosing
            // guard; any deeper `v-if` nested *inside* the loop body extends the
            // guard with extra `&& (...)` terms. The enclosing guard is therefore
            // the longest `&&`-separated prefix common to every direct expression
            // in the scope — conservative enough never to import a nested
            // branch's condition.
            let enclosing_guard: Option<String> = ctx
                .expressions_by_scope
                .get(&scope_id)
                .filter(|_| ctx.check_options.check_template_bindings)
                .and_then(|exprs| common_vif_guard_prefix(exprs));
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
                    &vfor_inner_indent,
                );
            }

            // Recursively generate child scopes inside this closure
            profile!(
                "canon.virtual_ts.child_scopes",
                generate_child_scopes(ts, mappings, ctx, scope_id, &vfor_inner_indent)
            );

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

            if data.target_component.is_some() {
                let event_types = generate_component_event_types(
                    ts,
                    ctx.summary,
                    data,
                    scope,
                    ctx.template_prop_names,
                    ctx.template_syntax_quirks,
                    indent,
                )
                .expect("component event handler should have a target component");
                let event_type = event_types.event_type;
                let args_type = event_types.args_type;
                let listener_type = event_types.listener_type;
                // Type the listener against the FULL emit argument tuple so
                // multi-arg emits keep every parameter (#1512). When the emit
                // signature stays unresolved (`unknown[]`, e.g. a fallthrough
                // DOM event on a component), fall back to the single `$event`
                // parameter so those handlers keep type-checking.
                append!(
                    *ts,
                    "{indent}type {listener_type} = unknown[] extends {args_type} ? (($event: {event_type}) => unknown) : ((...args: {args_type}) => unknown);\n",
                );
                // Receive every listener argument via a rest parameter typed by
                // `Parameters<listener>` (always a tuple, so the spread targets a
                // rest parameter and avoids TS2556). `$event` stays bound to the
                // first element for handlers/expressions that reference it.
                append!(
                    *ts,
                    "{indent}((...__vize_args: Parameters<{listener_type}>) => {{\n",
                );
                append!(
                    *ts,
                    "{inner_indent}const $event = __vize_args[0] as {event_type}; void $event;\n",
                );

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
                            event_listener_type: Some(listener_type.as_str()),
                            template_prop_names: ctx.template_prop_names,
                            template_offset: ctx.template_offset,
                            indent: &inner_indent,
                        },
                    )
                );

                append!(
                    *ts,
                    "{indent}}})(...({{}} as Parameters<{listener_type}>));\n",
                );
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
                            event_listener_type: None,
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

/// Compute the v-if guard active for a v-for *element* from the template
/// expressions recorded in its scope.
///
/// Every expression carries the joined `v-if` guard (`(a) && (b) && ...`)
/// active where it appears. For a v-for element, its own bindings/interpolations
/// share the element's enclosing guard, while expressions under a `v-if` nested
/// *inside* the loop body extend that guard with further `&& (...)` terms. The
/// enclosing guard is therefore the longest `&&`-separated prefix common to
/// every expression in the scope: it can never include a nested branch's
/// condition, and is `None` when any expression is unguarded (i.e. the v-for is
/// not inside a `v-if`). Returns the re-joined guard string when non-empty.
fn common_vif_guard_prefix(exprs: &[&vize_croquis::TemplateExpression]) -> Option<String> {
    let mut iter = exprs.iter();
    // Seed the common prefix with the first expression's guard terms; an
    // unguarded expression immediately rules out any common guard.
    let first = iter.next()?.vif_guard.as_ref()?;
    let mut common: Vec<&str> = split_guard_terms(first.as_str());

    for expr in iter {
        let guard = expr.vif_guard.as_ref()?;
        let terms = split_guard_terms(guard.as_str());
        let shared = common
            .iter()
            .zip(terms.iter())
            .take_while(|(a, b)| a == b)
            .count();
        common.truncate(shared);
        if common.is_empty() {
            return None;
        }
    }

    (!common.is_empty()).then(|| String::from(common.join(" && ").as_str()))
}

/// Split a joined v-if guard into its top-level ` && `-separated terms.
///
/// The drawer joins each branch condition — already wrapped as `(cond)` or
/// `!(cond)` — with ` && `, so a single condition may itself contain ` && `
/// inside its parentheses (`v-if="a && b"` becomes the term `(a && b)`). The
/// split must therefore only break on the ` && ` joiner at paren depth zero so
/// such conditions stay intact.
fn split_guard_terms(guard: &str) -> Vec<&str> {
    let bytes = guard.as_bytes();
    let mut terms = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'&' if depth == 0
                && bytes.get(index + 1) == Some(&b'&')
                && index >= 1
                && bytes[index - 1] == b' '
                && bytes.get(index + 2) == Some(&b' ') =>
            {
                terms.push(guard[start..index - 1].trim());
                index += 3;
                start = index;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    terms.push(guard[start..].trim());
    terms
}
