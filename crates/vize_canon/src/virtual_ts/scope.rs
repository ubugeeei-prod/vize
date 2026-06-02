//! Scope closure generation for virtual TypeScript.
//!
//! Generates TypeScript closures that mirror Vue's template scope hierarchy,
//! including v-for, v-slot, and event handler scopes. Uses recursive
//! tree-based generation so nested scopes are properly contained.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::profile;

use vize_croquis::{
    Croquis, EventHandlerScopeData, Scope, ScopeData, ScopeId, ScopeKind, analysis::ComponentUsage,
    analyzer::extract_identifiers_oxc, naming::to_pascal_case,
};

use super::{
    expressions::{generate_component_prop_checks, generate_expression},
    helpers::{
        generated_text_range, get_dom_event_type, to_camel_case, to_safe_identifier,
        to_safe_identifier_fragment,
    },
    types::{VirtualTsCheckOptions, VirtualTsOptions, VizeMapping},
};
use vize_carton::append;
use vize_carton::cstr;

/// Context for recursive scope generation, bundling shared parameters.
pub(crate) struct ScopeGenContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) template_prop_names: &'a FxHashSet<String>,
    pub(crate) template_offset: u32,
    pub(crate) check_options: VirtualTsCheckOptions,
}

/// Context for recursive component prop checks inside v-for scopes.
pub(crate) struct VForPropsContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) components_by_scope: &'a FxHashMap<u32, Vec<(usize, &'a ComponentUsage)>>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) template_prop_names: &'a FxHashSet<String>,
    pub(crate) template_offset: u32,
}

struct EventHandlerExprContext<'a> {
    expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    data: &'a EventHandlerScopeData,
    event_type: &'a str,
    template_offset: u32,
    indent: &'a str,
}

/// Generate scope closures from Croquis scope chain.
/// Uses recursive tree-based generation so nested v-for/v-slot scopes
/// are properly contained within their parent closures.
pub(crate) fn generate_scope_closures(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    check_options: VirtualTsCheckOptions,
    options: &VirtualTsOptions,
) {
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
                        summary.scopes.iter().any(|s| {
                            s.id == pid && matches!(s.kind, ScopeKind::VFor | ScopeKind::VSlot)
                        })
                    })
                })
                .map(|scope| scope.id)
                .collect()
        });

    if check_options.check_template_bindings {
        profile!(
            "canon.virtual_ts.instance_global_refs",
            generate_instance_global_refs(ts, mappings, summary, template_offset, options)
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
                for expr in exprs {
                    profile!(
                        "canon.virtual_ts.generate_expression",
                        generate_expression(
                            ts,
                            mappings,
                            expr,
                            template_prop_names,
                            template_offset,
                            "  "
                        )
                    );
                }
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
                summary,
                &children_map,
                template_prop_names,
                template_offset,
            )
        );
    }
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

    let mut seen_names: FxHashSet<&str> = FxHashSet::default();
    let mut emitted_header = false;
    for undef in &summary.undefined_refs {
        if !seen_names.insert(undef.name.as_str()) {
            continue;
        }
        if is_template_instance_global_name(undef.name.as_str()) {
            continue;
        }
        // Skip names that match type exports (these are type-level, not value-level)
        if type_export_names.contains(undef.name.as_str()) {
            continue;
        }

        let src_start = (template_offset + undef.offset) as usize;
        let src_end = src_start + undef.name.len();

        if !emitted_header {
            ts.push_str("\n  // Undefined references from template:\n");
            emitted_header = true;
        }

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
            sub_spans: Vec::new(),
        });
        append!(
            *ts,
            "  // @vize-map: {gen_name_start}:{gen_name_end} -> {src_start}:{src_end}\n",
        );
    }
}

fn generate_instance_global_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
    options: &VirtualTsOptions,
) {
    if summary.undefined_refs.is_empty() && summary.template_expressions.is_empty() {
        return;
    }

    let mut emitter = InstanceGlobalRefsEmitter::new(ts, mappings, summary, options);
    for undef in &summary.undefined_refs {
        let src_start = (template_offset + undef.offset) as usize;
        let src_end = src_start + undef.name.len();
        emitter.emit(undef.name.as_str(), src_start, src_end);
    }

    for expr in &summary.template_expressions {
        for ident in extract_identifiers_oxc(expr.content.as_str()) {
            let name = ident.as_str();
            let Some(relative_offset) = expr.content.find(name) else {
                continue;
            };
            let src_start = (template_offset + expr.start) as usize + relative_offset;
            let src_end = src_start + name.len();
            emitter.emit(name, src_start, src_end);
        }
    }
}

struct InstanceGlobalRefsEmitter<'a> {
    ts: &'a mut String,
    mappings: &'a mut Vec<VizeMapping>,
    options: &'a VirtualTsOptions,
    type_export_names: FxHashSet<&'a str>,
    seen_names: FxHashSet<String>,
    emitted_header: bool,
}

impl<'a> InstanceGlobalRefsEmitter<'a> {
    fn new(
        ts: &'a mut String,
        mappings: &'a mut Vec<VizeMapping>,
        summary: &'a Croquis,
        options: &'a VirtualTsOptions,
    ) -> Self {
        Self {
            ts,
            mappings,
            options,
            type_export_names: summary
                .type_exports
                .iter()
                .map(|te| te.name.as_str())
                .collect(),
            seen_names: FxHashSet::default(),
            emitted_header: false,
        }
    }

    fn emit(&mut self, name: &str, src_start: usize, src_end: usize) {
        if !is_template_instance_global_name(name)
            || self.type_export_names.contains(name)
            || is_declared_template_context_name(name, self.options)
            || !self.seen_names.insert(name.into())
        {
            return;
        }

        if !self.emitted_header {
            self.ts
                .push_str("\n  // Instance globals from ComponentPublicInstance:\n");
            self.ts.push_str(
                "  type __VizeInstanceGlobal<K extends string> = K extends keyof __Ctx ? __Ctx[K] : any;\n",
            );
            self.emitted_header = true;
        }

        let gen_start = self.ts.len();
        let stmt = cstr!("  const {name}: __VizeInstanceGlobal<'{name}'> = undefined as any;\n");
        let gen_name_start = gen_start + stmt.find(name).unwrap_or(0);
        let gen_name_end = gen_name_start + name.len();

        self.ts.push_str(&stmt);
        self.mappings.push(VizeMapping {
            gen_range: gen_name_start..gen_name_end,
            src_range: src_start..src_end,
            sub_spans: Vec::new(),
        });
        append!(
            *self.ts,
            "  // @vize-map: {gen_name_start}:{gen_name_end} -> {src_start}:{src_end}\n",
        );
    }
}

fn is_template_instance_global_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('$') else {
        return false;
    };
    !rest.is_empty()
        && rest
            .chars()
            .all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}

fn is_declared_template_context_name(name: &str, options: &VirtualTsOptions) -> bool {
    matches!(name, "$attrs" | "$slots" | "$refs" | "$emit" | "$event")
        || options
            .template_globals
            .iter()
            .any(|global| global.name.as_str() == name)
        || options
            .css_modules
            .iter()
            .any(|module_name| module_name.as_str() == name)
}

/// Type annotation for a `v-slot` scope's props. When the slot is on a child
/// component (`component` is `Some`), the props are inferred from that child's
/// `$slots[name]` parameter (its `defineSlots`), so misuse raises a real
/// diagnostic (#764). Otherwise — and whenever the child has no typed slot —
/// it falls back to `any` so untyped or built-in slot hosts never produce a
/// false positive.
fn slot_props_type(component: Option<&str>, slot_name: &str) -> String {
    match component {
        Some(component) => {
            let component_ref = to_safe_identifier(component);
            cstr!(
                "typeof {component_ref} extends {{ new (): {{ $slots: infer __S }} }} ? (__S extends {{ \"{slot_name}\"?: (props: infer __P, ...args: any[]) => any }} ? __P : any) : any"
            )
        }
        None => "any".into(),
    }
}

fn append_v_for_comment(ts: &mut String, indent: &str, label: &str, alias: &str, source: &str) {
    append!(*ts, "\n{indent}// {label}: {alias} in ");
    for c in source.chars() {
        if c == '\n' || c == '\r' {
            ts.push(' ');
        } else {
            ts.push(c);
        }
    }
    ts.push('\n');
}

/// Emit the opening of a v-for scope as
/// `__vForList(source).forEach(([value, key, index]) => {`.
///
/// The overloaded `__vForList` helper types the destructured tuple from the
/// source kind: arrays/iterables/numbers/strings keep a numeric `key`, while an
/// object source yields `value: T[keyof T]` and `key: keyof T` (matching
/// vue-tsc) instead of the old array-only `(source).forEach` assumption that
/// mis-typed objects and raised spurious TS2339/TS2537. The source expression is
/// passed through verbatim so any `as Type` assertion flows into the helper.
fn emit_v_for_loop_open(
    ts: &mut String,
    indent: &str,
    value_alias: &str,
    key_alias: Option<&str>,
    index_alias: Option<&str>,
    source: &str,
) {
    append!(*ts, "{indent}__vForList({source}).forEach(([{value_alias}");
    if let Some(key) = key_alias {
        append!(*ts, ", {key}");
    } else if index_alias.is_some() {
        // Keep the index in the third tuple slot even when no key alias is bound.
        ts.push_str(", _key");
    }
    if let Some(index) = index_alias {
        append!(*ts, ", {index}");
    }
    ts.push_str("]) => {\n");
}

/// Generate component props type checks (scope-aware).
/// Type declarations are at template level, value checks are in their scope.
fn generate_component_props(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    children_map: &FxHashMap<u32, Vec<ScopeId>>,
    template_prop_names: &FxHashSet<String>,
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

    // Helper types for the generic functional prop-check path (#775). A
    // `<script setup generic="T">` child exposes `__vizeCheck<T>(props)` on its
    // default export; `__VizePropChecker<C>` extracts that generic signature so
    // the parent can call it and let TS infer `T` from the passed props. When
    // the child is non-generic (plain construct signature), a built-in / library
    // component, or `any`, it falls back to `(props: any) => void`, a no-op that
    // never reports, so only generic components take the new path and the
    // well-tested `typeof Comp extends { $props }` extraction below is preserved.
    let any_dynamic_props = summary.component_usages.iter().any(|usage| {
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
    }

    for (idx, usage) in summary.component_usages.iter().enumerate() {
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

        let src_start = (template_offset + usage.start) as usize;
        let src_end = (template_offset + usage.end) as usize;

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
                    "  type __{component_type_name}_{idx}_prop_{safe_prop_name} = __{component_type_name}_Props_{idx} extends {{ '{camel_prop_name}'?: infer T }} ? T : __{component_type_name}_Props_{idx} extends {{ '{camel_prop_name}': infer T }} ? T : unknown;\n",
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
        profile!(
            "canon.virtual_ts.component_prop_checks",
            generate_component_prop_checks(
                ts,
                mappings,
                usage,
                idx,
                template_prop_names,
                template_offset,
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
            children_map,
            template_prop_names,
            template_offset,
        };
        profile!(
            "canon.virtual_ts.closure_component_props",
            generate_closure_component_props_recursive(ts, mappings, &props_ctx, scope, "  ")
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
                for expr in exprs {
                    profile!(
                        "canon.virtual_ts.generate_expression",
                        generate_expression(
                            ts,
                            mappings,
                            expr,
                            ctx.template_prop_names,
                            ctx.template_offset,
                            &inner_indent
                        )
                    );
                }
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
            let props_type = slot_props_type(data.component.as_deref(), data.name.as_str());
            append!(
                *ts,
                "{indent}void function _slot_{safe_slot_name}({props_pattern}: {props_type}) {{\n",
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
                for expr in exprs {
                    profile!(
                        "canon.virtual_ts.generate_expression",
                        generate_expression(
                            ts,
                            mappings,
                            expr,
                            ctx.template_prop_names,
                            ctx.template_offset,
                            &inner_indent
                        )
                    );
                }
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
                    cstr!("\"{on_handler}\"")
                } else {
                    on_handler
                };

                // Type alias (block-scoped in TypeScript)
                // Include scope_id to deduplicate when same component+event appears multiple times
                append!(
                    *ts,
                    "{indent}type __{component_type_name}_{scope_id}_{safe_event_name}_event = typeof {component_ref} extends {{ new (): {{ $props: infer __P }} }}\n",
                );
                append!(
                    *ts,
                    "{indent}  ? __P extends {{ {prop_key}?: (arg: infer __A, ...rest: any[]) => any }} ? __A : unknown\n",
                );
                append!(
                    *ts,
                    "{indent}  : typeof {component_ref} extends (props: infer __P) => any\n",
                );
                append!(
                    *ts,
                    "{indent}    ? __P extends {{ {prop_key}?: (arg: infer __A, ...rest: any[]) => any }} ? __A : unknown\n",
                );
                append!(*ts, "{indent}    : unknown;\n");

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
                for expr in exprs {
                    profile!(
                        "canon.virtual_ts.generate_expression",
                        generate_expression(
                            ts,
                            mappings,
                            expr,
                            ctx.template_prop_names,
                            ctx.template_offset,
                            indent
                        )
                    );
                }
            }
        }
    }
}

/// Generate event handler expressions inside a closure.
fn generate_event_handler_expressions(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    scope_id: u32,
    ctx: &EventHandlerExprContext<'_>,
) {
    if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id) {
        for expr in exprs {
            let content = expr.content.as_str();
            let is_implicit_reference =
                ctx.data.has_implicit_event && is_callable_handler_reference(content);
            let src_start = (ctx.template_offset + expr.start) as usize;
            let src_end = (ctx.template_offset + expr.end) as usize;

            let gen_stmt_start = ts.len();
            if is_implicit_reference {
                let handler_name = cstr!("__vize_handler_{scope_id}_{}", expr.start);
                append!(
                    *ts,
                    "{indent}const {handler_name} = ((handler: ($event: {event_type}) => unknown) => handler)(({content}));\n",
                    indent = ctx.indent,
                    event_type = ctx.event_type,
                );
                append!(
                    *ts,
                    "{indent}{handler_name}($event);  // handler expression\n",
                    indent = ctx.indent,
                );
            } else {
                append!(
                    *ts,
                    "{indent}{content};  // handler expression\n",
                    indent = ctx.indent
                );
            }
            let gen_stmt_end = ts.len();
            mappings.push(VizeMapping {
                gen_range: if is_implicit_reference {
                    gen_stmt_start..gen_stmt_end
                } else {
                    generated_text_range(&ts[gen_stmt_start..gen_stmt_end], content, gen_stmt_start)
                },
                src_range: src_start..src_end,
                sub_spans: Vec::new(),
            });
            append!(
                *ts,
                "{indent}// @vize-map: handler -> {src_start}:{src_end}\n",
                indent = ctx.indent,
            );
        }
    }
}

fn is_callable_handler_reference(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }

    trimmed.split('.').all(is_identifier_segment)
}

fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch == '$' || ch.is_alphanumeric())
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
            append_v_for_comment(
                ts,
                indent,
                "Component props in v-for scope",
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
            ts.push_str("});\n");
        }
        ScopeData::VSlot(data) => {
            let props_pattern = data.props_pattern.as_deref().unwrap_or("slotProps");
            let safe_slot_name = to_safe_identifier_fragment(data.name.as_str());
            let props_type = slot_props_type(data.component.as_deref(), data.name.as_str());
            append!(
                *ts,
                "\n{indent}// Component props in v-slot scope: #{}\n",
                data.name
            );
            append!(
                *ts,
                "{indent}void function _slot_props_{safe_slot_name}({props_pattern}: {props_type}) {{\n",
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
