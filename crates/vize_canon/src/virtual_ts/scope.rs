//! Scope closure generation for virtual TypeScript.
//!
//! Generates TypeScript closures that mirror Vue's template scope hierarchy,
//! including v-for, v-slot, and event handler scopes. Uses recursive
//! tree-based generation so nested scopes are properly contained.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;

use vize_croquis::{
    analysis::ComponentUsage, naming::to_pascal_case, Croquis, EventHandlerScopeData, Scope,
    ScopeData, ScopeId, ScopeKind,
};

use super::{
    expressions::{generate_component_prop_checks, generate_expression},
    helpers::{
        generated_text_range, get_dom_event_type, strip_as_assertion, to_camel_case,
        to_safe_identifier,
    },
    types::VizeMapping,
};
use vize_carton::append;
use vize_carton::cstr;

/// Context for recursive scope generation, bundling shared parameters.
pub(crate) struct ScopeGenContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) template_offset: u32,
}

/// Context for recursive component prop checks inside v-for scopes.
pub(crate) struct VForPropsContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) components_by_scope: &'a FxHashMap<u32, Vec<(usize, &'a ComponentUsage)>>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) template_offset: u32,
}

/// Generate scope closures from Croquis scope chain.
/// Uses recursive tree-based generation so nested v-for/v-slot scopes
/// are properly contained within their parent closures.
pub(crate) fn generate_scope_closures(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
) {
    // Group expressions by scope_id
    let mut expressions_by_scope: FxHashMap<u32, Vec<_>> = FxHashMap::default();
    for expr in &summary.template_expressions {
        expressions_by_scope
            .entry(expr.scope_id.as_u32())
            .or_default()
            .push(expr);
    }

    // Build scope tree: parent_scope_id -> Vec<child ScopeId>
    let mut children_map: FxHashMap<u32, Vec<ScopeId>> = FxHashMap::default();
    for scope in summary.scopes.iter() {
        if let Some(parent_id) = scope.parent() {
            children_map
                .entry(parent_id.as_u32())
                .or_default()
                .push(scope.id);
        }
    }

    // Determine which scopes are nested inside a closure scope (VFor/VSlot).
    // These will be generated recursively inside their parent, not at top level.
    let nested_scope_ids: FxHashSet<ScopeId> = summary
        .scopes
        .iter()
        .filter(|scope| {
            scope.parent().is_some_and(|pid| {
                summary
                    .scopes
                    .iter()
                    .any(|s| s.id == pid && matches!(s.kind, ScopeKind::VFor | ScopeKind::VSlot))
            })
        })
        .map(|scope| scope.id)
        .collect();

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
            if let Some(exprs) = expressions_by_scope.get(&scope_id) {
                for expr in exprs {
                    generate_expression(ts, mappings, expr, template_offset, "  ");
                }
            }
            continue;
        }

        let ctx = ScopeGenContext {
            summary,
            expressions_by_scope: &expressions_by_scope,
            children_map: &children_map,
            template_offset,
        };
        generate_scope_node(ts, mappings, &ctx, scope, "  ");
    }

    // Handle undefined references
    generate_undefined_refs(ts, mappings, summary, template_offset);

    // Generate component props type checks (scope-aware)
    generate_component_props(ts, mappings, summary, &children_map, template_offset);
}

/// Handle undefined references from template.
fn generate_undefined_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
) {
    if summary.undefined_refs.is_empty() {
        return;
    }

    // Collect type export names to exclude from undefined refs
    let type_export_names: FxHashSet<&str> = summary
        .type_exports
        .iter()
        .map(|te| te.name.as_str())
        .collect();

    ts.push_str("\n  // Undefined references from template:\n");
    let mut seen_names: FxHashSet<&str> = FxHashSet::default();
    for undef in &summary.undefined_refs {
        if !seen_names.insert(undef.name.as_str()) {
            continue;
        }
        // Skip names that match type exports (these are type-level, not value-level)
        if type_export_names.contains(undef.name.as_str()) {
            continue;
        }

        let src_start = (template_offset + undef.offset) as usize;
        let src_end = src_start + undef.name.len();

        let gen_start = ts.len();
        // Use void expression to reference the name without creating an unused variable
        let expr_code = cstr!("  void ({});\n", undef.name);
        let name_offset = expr_code.find(undef.name.as_str()).unwrap_or(0);
        let gen_name_start = gen_start + name_offset;
        let gen_name_end = gen_name_start + undef.name.len();

        ts.push_str(&expr_code);
        mappings.push(VizeMapping {
            gen_range: gen_name_start..gen_name_end,
            src_range: src_start..src_end,
        });
        append!(
            *ts,
            "  // @vize-map: {gen_name_start}:{gen_name_end} -> {src_start}:{src_end}\n",
        );
    }
}

/// Generate component props type checks (scope-aware).
/// Type declarations are at template level, value checks are in their scope.
fn generate_component_props(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    children_map: &FxHashMap<u32, Vec<ScopeId>>,
    template_offset: u32,
) {
    if summary.component_usages.is_empty() {
        return;
    }

    // Group component usages by scope_id
    let mut components_by_scope: FxHashMap<u32, Vec<(usize, &ComponentUsage)>> =
        FxHashMap::default();
    for (idx, usage) in summary.component_usages.iter().enumerate() {
        components_by_scope
            .entry(usage.scope_id.as_u32())
            .or_default()
            .push((idx, usage));
    }

    // Emit type declarations only for components with dynamic props
    // (TypeScript type aliases cannot be inside function bodies)
    ts.push_str("\n  // Component props type declarations\n");
    for (idx, usage) in summary.component_usages.iter().enumerate() {
        let component_name = &usage.name;

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

        let src_start = (template_offset + usage.start) as usize;
        let src_end = (template_offset + usage.end) as usize;

        append!(*ts, "  // @vize-map: component -> {src_start}:{src_end}\n",);
        append!(
            *ts,
            "  type __{component_name}_Props_{idx} = typeof {component_name} extends {{ new (): {{ $props: infer __P }} }} ? __P : (typeof {component_name} extends (props: infer __P) => any ? __P : {{}});\n",
        );

        for prop in &usage.props {
            if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
                continue;
            }
            if prop.value.is_some() && prop.is_dynamic {
                let camel_prop_name = to_camel_case(prop.name.as_str());
                let safe_prop_name = prop.name.replace('-', "_");
                append!(
                    *ts,
                    "  type __{component_name}_{idx}_prop_{safe_prop_name} = __{component_name}_Props_{idx} extends {{ '{camel_prop_name}'?: infer T }} ? T : __{component_name}_Props_{idx} extends {{ '{camel_prop_name}': infer T }} ? T : unknown;\n",
                );
            }
        }
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
                    summary
                        .scopes
                        .iter()
                        .find(|p| p.id == pid)
                        .is_none_or(|p| !matches!(p.kind, ScopeKind::VFor | ScopeKind::VSlot))
                })
        })
        .map(|s| s.id.as_u32())
        .collect();

    ts.push_str("\n  // Component props value checks (template scope)\n");
    for (idx, usage) in summary.component_usages.iter().enumerate() {
        if closure_scope_ids.contains(&usage.scope_id.as_u32()) {
            continue; // Will be emitted inside v-for/v-slot scope
        }
        generate_component_prop_checks(ts, mappings, usage, idx, template_offset, "  ");
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
            children_map,
            template_offset,
        };
        generate_closure_component_props_recursive(ts, mappings, &props_ctx, scope, "  ");
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
            append!(
                *ts,
                "\n{indent}// v-for scope: {} in {}\n",
                data.value_alias,
                data.source
            );

            // Strip TypeScript `as Type` assertion from v-for source expression.
            // e.g., "(expr) as OptionSponsor[]" -> "(expr)" with type annotation
            let (source_expr, type_annotation) = strip_as_assertion(&data.source);

            // Detect numeric literal (e.g., `v-for="n in 4"`)
            let is_numeric = source_expr.trim().parse::<u64>().is_ok();

            let is_simple_identifier = source_expr.chars().all(|c| c.is_alphanumeric() || c == '_');
            let element_type = if is_numeric {
                "number".into()
            } else if let Some(ref ta) = type_annotation {
                // Use the asserted type's element type
                cstr!("{ta}[number]")
            } else if is_simple_identifier {
                cstr!("typeof {source_expr}[number]")
            } else {
                "any".into()
            };

            if is_numeric {
                append!(
                    *ts,
                    "{indent}(Array.from({{length: {source_expr}}}, (_, __i) => __i + 1)).forEach(({}: {element_type}",
                    data.value_alias,
                );
            } else {
                append!(
                    *ts,
                    "{indent}({source_expr}).forEach(({}: {element_type}",
                    data.value_alias,
                );
            }

            if let Some(ref key) = data.key_alias {
                append!(*ts, ", {key}: number");
            }
            if let Some(ref index) = data.index_alias {
                if data.key_alias.is_none() {
                    ts.push_str(", _key: number");
                }
                append!(*ts, ", {index}: number");
            }

            ts.push_str(") => {\n");

            // Mark v-for variables as used to avoid TS6133
            append!(*ts, "{inner_indent}void {};\n", data.value_alias);
            if let Some(ref key) = data.key_alias {
                append!(*ts, "{inner_indent}void {key};\n");
            }
            if let Some(ref index) = data.index_alias {
                append!(*ts, "{inner_indent}void {index};\n");
            }

            // Generate expressions in this scope
            if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id) {
                for expr in exprs {
                    generate_expression(ts, mappings, expr, ctx.template_offset, &inner_indent);
                }
            }

            // Recursively generate child scopes inside this closure
            generate_child_scopes(ts, mappings, ctx, scope_id, &inner_indent);

            ts.push_str(indent);
            ts.push_str("});\n");
        }
        ScopeData::VSlot(data) => {
            append!(*ts, "\n{indent}// v-slot scope: #{}\n", data.name);

            let props_pattern = data.props_pattern.as_deref().unwrap_or("slotProps");
            append!(
                *ts,
                "{indent}void function _slot_{}({props_pattern}: any) {{\n",
                data.name,
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

            if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id) {
                for expr in exprs {
                    generate_expression(ts, mappings, expr, ctx.template_offset, &inner_indent);
                }
            }

            // Recursively generate child scopes inside this closure
            generate_child_scopes(ts, mappings, ctx, scope_id, &inner_indent);

            ts.push_str(indent);
            ts.push_str("};\n");
        }
        ScopeData::EventHandler(data) => {
            append!(*ts, "\n{indent}// @{} handler\n", data.event_name);

            let safe_event_name = to_safe_identifier(data.event_name.as_str());

            if let Some(ref component_name) = data.target_component {
                let pascal_event = to_pascal_case(data.event_name.as_str());
                let on_handler = cstr!("on{pascal_event}");

                let prop_key = if on_handler.contains(':') {
                    cstr!("\"{on_handler}\"")
                } else {
                    on_handler
                };

                // Type alias (block-scoped in TypeScript)
                // Include scope_id to deduplicate when same component+event appears multiple times
                append!(
                    *ts,
                    "{indent}type __{component_name}_{scope_id}_{safe_event_name}_event = typeof {component_name} extends {{ new (): {{ $props: infer __P }} }}\n",
                );
                append!(
                    *ts,
                    "{indent}  ? __P extends {{ {prop_key}?: (arg: infer __A, ...rest: any[]) => any }} ? __A : unknown\n",
                );
                append!(
                    *ts,
                    "{indent}  : typeof {component_name} extends (props: infer __P) => any\n",
                );
                append!(
                    *ts,
                    "{indent}    ? __P extends {{ {prop_key}?: (arg: infer __A, ...rest: any[]) => any }} ? __A : unknown\n",
                );
                append!(*ts, "{indent}    : unknown;\n");

                let event_type = cstr!("__{component_name}_{scope_id}_{safe_event_name}_event");
                append!(*ts, "{indent}(($event: {event_type}) => {{\n");

                generate_event_handler_expressions(
                    ts,
                    mappings,
                    ctx.expressions_by_scope,
                    scope_id,
                    data,
                    ctx.template_offset,
                    &inner_indent,
                );

                append!(*ts, "{indent}}})({{}} as {event_type});\n");
            } else {
                let event_type = get_dom_event_type(data.event_name.as_str());
                append!(*ts, "{indent}(($event: {event_type}) => {{\n");

                generate_event_handler_expressions(
                    ts,
                    mappings,
                    ctx.expressions_by_scope,
                    scope_id,
                    data,
                    ctx.template_offset,
                    &inner_indent,
                );

                append!(*ts, "{indent}}})({{}} as {event_type});\n");
            }
        }
        _ => {
            if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id) {
                for expr in exprs {
                    generate_expression(ts, mappings, expr, ctx.template_offset, indent);
                }
            }
        }
    }
}

/// Generate event handler expressions inside a closure.
fn generate_event_handler_expressions(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expressions_by_scope: &FxHashMap<u32, Vec<&vize_croquis::TemplateExpression>>,
    scope_id: u32,
    data: &EventHandlerScopeData,
    template_offset: u32,
    indent: &str,
) {
    if let Some(exprs) = expressions_by_scope.get(&scope_id) {
        for expr in exprs {
            let content = expr.content.as_str();
            let is_simple_identifier = content
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '$');

            let src_start = (template_offset + expr.start) as usize;
            let src_end = (template_offset + expr.end) as usize;

            let gen_stmt_start = ts.len();
            if data.has_implicit_event && is_simple_identifier && !content.is_empty() {
                append!(*ts, "{indent}({content} as (...args: any[]) => any)($event);  // handler expression\n",);
            } else {
                append!(*ts, "{indent}{content};  // handler expression\n");
            }
            let gen_stmt_end = ts.len();
            mappings.push(VizeMapping {
                gen_range: generated_text_range(
                    &ts[gen_stmt_start..gen_stmt_end],
                    content,
                    gen_stmt_start,
                ),
                src_range: src_start..src_end,
            });
            append!(
                *ts,
                "{indent}// @vize-map: handler -> {src_start}:{src_end}\n",
            );
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
            if let Some(child_scope) = ctx.summary.scopes.get_scope(child_id) {
                if matches!(
                    child_scope.kind,
                    ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::EventHandler
                ) {
                    generate_scope_node(ts, mappings, ctx, child_scope, indent);
                }
            }
        }
    }
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
    let inner_indent = cstr!("{indent}  ");

    match scope.data() {
        ScopeData::VFor(data) => {
            let (source_expr, type_annotation) = strip_as_assertion(&data.source);

            let is_numeric = source_expr.trim().parse::<u64>().is_ok();

            let is_simple_identifier = source_expr.chars().all(|c| c.is_alphanumeric() || c == '_');
            let element_type = if is_numeric {
                "number".into()
            } else if let Some(ref ta) = type_annotation {
                cstr!("{ta}[number]")
            } else if is_simple_identifier {
                cstr!("typeof {source_expr}[number]")
            } else {
                "any".into()
            };

            append!(
                *ts,
                "\n{indent}// Component props in v-for scope: {} in {}\n",
                data.value_alias,
                data.source
            );
            if is_numeric {
                append!(
                    *ts,
                    "{indent}(Array.from({{length: {source_expr}}}, (_, __i) => __i + 1)).forEach(({}: {element_type}",
                    data.value_alias,
                );
            } else {
                append!(
                    *ts,
                    "{indent}({source_expr}).forEach(({}: {element_type}",
                    data.value_alias,
                );
            }
            if let Some(ref key) = data.key_alias {
                append!(*ts, ", {key}: number");
            }
            if let Some(ref index) = data.index_alias {
                if data.key_alias.is_none() {
                    ts.push_str(", _key: number");
                }
                append!(*ts, ", {index}: number");
            }
            ts.push_str(") => {\n");

            // Mark v-for variables as used to avoid TS6133
            append!(*ts, "{inner_indent}void {};\n", data.value_alias);
            if let Some(ref key) = data.key_alias {
                append!(*ts, "{inner_indent}void {key};\n");
            }
            if let Some(ref index) = data.index_alias {
                append!(*ts, "{inner_indent}void {index};\n");
            }

            // Emit component prop checks for this scope
            if let Some(usages) = ctx.components_by_scope.get(&scope_id) {
                for &(idx, usage) in usages {
                    generate_component_prop_checks(
                        ts,
                        mappings,
                        usage,
                        idx,
                        ctx.template_offset,
                        &inner_indent,
                    );
                }
            }

            // Recursively handle child closure scopes (v-for and v-slot)
            if let Some(child_ids) = ctx.children_map.get(&scope_id) {
                for &child_id in child_ids {
                    if let Some(child_scope) = ctx.summary.scopes.get_scope(child_id) {
                        if matches!(child_scope.kind, ScopeKind::VFor | ScopeKind::VSlot) {
                            generate_closure_component_props_recursive(
                                ts,
                                mappings,
                                ctx,
                                child_scope,
                                &inner_indent,
                            );
                        }
                    }
                }
            }

            ts.push_str(indent);
            ts.push_str("});\n");
        }
        ScopeData::VSlot(data) => {
            let props_pattern = data.props_pattern.as_deref().unwrap_or("slotProps");
            append!(
                *ts,
                "\n{indent}// Component props in v-slot scope: #{}\n",
                data.name
            );
            append!(
                *ts,
                "{indent}void function _slot_props_{}({props_pattern}: any) {{\n",
                data.name,
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
                    generate_component_prop_checks(
                        ts,
                        mappings,
                        usage,
                        idx,
                        ctx.template_offset,
                        &inner_indent,
                    );
                }
            }

            // Recursively handle child closure scopes (v-for and v-slot)
            if let Some(child_ids) = ctx.children_map.get(&scope_id) {
                for &child_id in child_ids {
                    if let Some(child_scope) = ctx.summary.scopes.get_scope(child_id) {
                        if matches!(child_scope.kind, ScopeKind::VFor | ScopeKind::VSlot) {
                            generate_closure_component_props_recursive(
                                ts,
                                mappings,
                                ctx,
                                child_scope,
                                &inner_indent,
                            );
                        }
                    }
                }
            }

            ts.push_str(indent);
            ts.push_str("};\n");
        }
        _ => {}
    }
}
