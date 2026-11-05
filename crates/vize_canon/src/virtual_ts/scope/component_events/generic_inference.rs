use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::{
    Croquis, EventHandlerScopeData, Scope,
    analysis::{ComponentUsage, PassedProp},
};

use crate::virtual_ts::{
    expressions::rewrite_reserved_template_binding, helpers::to_camel_case,
    scope::is_inline_callback_prop,
};

use crate::virtual_ts::expressions::{
    CallbackParameterRewrite, ComponentPropSource, PropSpan, append_props_literal,
    append_spread_value, callback_parameters_on_ignored_line_with_ranges, generated_prop_span,
    map_rewritten_template_binding, prop_entry_sub_spans_with_synthetic_value_gaps,
    prop_value_source_range,
};
use crate::virtual_ts::types::VizeMapping;

pub(super) struct EmitInferenceContext<'a, 'template> {
    pub(super) summary: &'a Croquis,
    pub(super) component_name: &'a str,
    pub(super) data: &'a EventHandlerScopeData,
    pub(super) scope: &'a Scope,
    pub(super) component_ref: &'a str,
    pub(super) component_type_name: &'a str,
    pub(super) safe_event_name: &'a str,
    pub(super) prop_key: &'a str,
    /// Alias that is `true` for a vue-tsc generic library export.
    pub(super) library: &'a str,
    pub(super) template_binding_access: &'a TemplateBindingAccess<'a>,
    pub(super) template_ast: Option<&'a vize_relief::RootNode<'template>>,
    pub(super) source_context: ComponentPropSource<'a>,
    pub(super) enclosing_vif_guard: Option<&'a str>,
    pub(super) event_inference_usages: &'a std::cell::RefCell<FxHashSet<usize>>,
    pub(super) indent: &'a str,
    pub(super) hoist_shared_preamble: bool,
    /// The module declares the gate aliases and this component's type is closed.
    pub(super) gated_inference: bool,
    /// Modern consumers use loose ownership for runtime args and callback context.
    pub(super) contextual_listener: bool,
}

/// Argument shape and listener-context ownership are independent facts.
pub(super) struct InferredEmitArgs {
    pub(super) arguments: String,
    pub(super) contextless_listener: Option<String>,
}

pub(super) fn generate_inferred_emit_args(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &EmitInferenceContext<'_, '_>,
) -> Option<InferredEmitArgs> {
    let (usage_idx, usage) =
        find_component_usage_for_event(ctx.summary, ctx.component_name, ctx.data, ctx.scope)?;
    let dynamic_alias =
        crate::virtual_ts::scope::is_owned_dynamic_component_alias(ctx.template_ast, &usage);
    let dynamic_component = dynamic_alias
        || crate::virtual_ts::scope::is_owned_named_dynamic_component(ctx.template_ast, &usage);
    // `:is` is not a passed prop. A listener-only `<component :is>` still has
    // to call the factory: a loose target returns `$LooseEmitProps`. Its marker
    // must remain distinct from an actual declared empty tuple. Other components need a valued
    // prop or a spread before that call infers anything.
    let has_inference_prop = !usage.spread_props.is_empty()
        || usage.props.iter().any(|prop| {
            !prop.name_is_dynamic
                && prop.name.as_str() != "key"
                && prop.name.as_str() != "ref"
                && prop.value.is_some()
        });
    if !has_inference_prop && !dynamic_component {
        return None;
    }

    let scope_id = ctx.scope.id.as_u32();
    let resolver_type = cstr!("__{}_{}_emit_resolver", ctx.component_type_name, usage_idx);
    let emit_props = cstr!("__vize_emit_props_{usage_idx}");
    let inferred_args = cstr!(
        "__{}_{}_{}_inferred_emit_args",
        ctx.component_type_name,
        scope_id,
        ctx.safe_event_name
    );
    let first_listener = ctx.event_inference_usages.borrow_mut().insert(usage_idx);
    let emit_props_factory = crate::virtual_ts::helpers::shared_types::EMIT_PROPS_FACTORY
        .select(ctx.hoist_shared_preamble);
    let guard = usage
        .vif_guard
        .as_ref()
        .map(|guard| {
            rewrite_reserved_template_binding(guard.as_str(), ctx.template_binding_access)
                .unwrap_or_else(|| guard.clone())
        })
        .and_then(|guard| {
            ctx.enclosing_vif_guard
                .and_then(|enclosing| {
                    crate::virtual_ts::scope::remove_enclosing_vif_guard_prefix(
                        guard.as_str(),
                        enclosing,
                    )
                })
                .or_else(|| ctx.enclosing_vif_guard.is_none().then_some(guard))
        });
    let guarded_call_indent = guard.as_ref().map(|_| cstr!("{}  ", ctx.indent));
    let call_indent = guarded_call_indent.as_deref().unwrap_or(ctx.indent);
    if first_listener
        && guard.is_none()
        && !dynamic_component
        && let Some(spread) = usage.spread_props.first()
    {
        let probe_type = cstr!(
            "__{}_{}_spread_instantiation_probe",
            ctx.component_type_name,
            usage_idx
        );
        append!(
            *ts,
            "{}type {probe_type}<P, A, K extends keyof P & keyof A = keyof P & keyof A> = {{ [Q in K]: A[Q] extends P[Q] ? true : false }}[K];\n",
            ctx.indent
        );
        append!(
            *ts,
            "{}(undefined as unknown as <A>(value: A) => {probe_type}<typeof {} extends abstract new (...args: any[]) => infer I ? I extends {{ $props: infer P }} ? P : {{}} : typeof {} extends (props: infer P, ...args: any[]) => any ? P : {{}}, A>)(\n",
            ctx.indent,
            ctx.component_ref,
            ctx.component_ref,
        );
        append!(
            *ts,
            "{}  // @ts-ignore Probe leaf errors are owned by the canonical component-props call.\n{}  ",
            ctx.indent,
            ctx.indent,
        );
        let (probe_argument_range, _) = append_spread_value(
            ts,
            spread.expression.as_str(),
            None,
            ctx.template_binding_access,
            &ctx.summary.scopes,
            usage.scope_id,
        );
        append!(*ts, "\n{});\n", ctx.indent);
        if let Some(src_range) = crate::virtual_ts::component_tag::source_range(
            ctx.source_context.template,
            ctx.source_context.offset,
            &usage,
        ) {
            mappings.push(VizeMapping {
                gen_range: probe_argument_range,
                src_range,
                sub_spans: Vec::new(),
            });
        }
    }
    let entries = if first_listener && usage.spread_props.is_empty() {
        inference_entries(&usage, ctx.template_binding_access)
    } else {
        Vec::new()
    };
    // An empty literal is not a gated candidate: the gate aliases are emitted
    // only when a valued prop exists, and a loose `:is` must keep the marker.
    let gated = has_inference_prop
        && ctx.gated_inference
        && dynamic_alias
        && guard.is_none()
        && usage.spread_props.is_empty()
        && entries
            .iter()
            .all(|entry| super::emit_props_gate::is_single_line(entry.value()));
    let (indent, library, component_ref) = (ctx.indent, ctx.library, ctx.component_ref);
    // A vue-tsc library export is called itself, so the bound props
    // instantiate its generics and its `__ctx` carries the listener props.
    if first_listener && !gated {
        let share = super::emit_resolver_share::ResolverShare {
            component_type_name: ctx.component_type_name,
            component_ref,
            factory: emit_props_factory,
            dynamic: dynamic_component,
            parent_scope: ctx.scope.parent().map(|parent| parent.as_u32()),
            block: super::super::open_chained_vif_guard_block(),
        };
        if let Some(canonical) = super::emit_resolver_share::alias_target(&share) {
            append!(*ts, "{indent}type {resolver_type} = {canonical};\n");
        } else {
            append!(
                *ts,
                "{indent}type {resolver_type} = {library} extends true ? (component: typeof {component_ref}) => typeof {component_ref} : {emit_props_factory}<typeof {component_ref}>;\n",
            );
            super::emit_resolver_share::record_inline(&share, resolver_type.as_str());
        }
    }
    if first_listener && let Some(guard) = guard.as_deref() {
        let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
        let guard = isolated.as_str();
        append!(*ts, "{}const {emit_props} = (() => {{\n", ctx.indent);
        append!(
            *ts,
            "{call_indent}// @ts-ignore Inference-only guard; authored binding checks own diagnostics.\n"
        );
        append!(
            *ts,
            "{call_indent}if ({guard}) return (undefined as unknown as {resolver_type})({})(",
            ctx.component_ref,
        );
    } else if first_listener {
        if dynamic_component || !usage.spread_props.is_empty() {
            append!(
                *ts,
                "{}// @ts-ignore Inference-only dynamic target/props; the canonical component-props call owns diagnostics.\n",
                ctx.indent
            );
        }
        if gated {
            super::emit_props_gate::append_gated_call(
                ts,
                indent,
                library,
                component_ref,
                &emit_props,
                usage_idx,
            );
        } else {
            append!(
                *ts,
                "{indent}const {emit_props} = (undefined as unknown as {resolver_type})({component_ref})(",
            );
        }
    }
    if first_listener {
        if !usage.spread_props.is_empty() {
            let literal_range = append_props_literal(
                ts,
                mappings,
                &usage,
                ctx.template_binding_access,
                ctx.source_context.inference_only(),
                call_indent,
            );
            ts.push_str(");\n");
            // Argument assignability errors cover the whole literal, rather than
            // one spread expression. Preserve the same authored tag owner as the
            // canonical props call; these inference rows remain diagnostics-only.
            if let Some(src_range) = crate::virtual_ts::component_tag::source_range(
                ctx.source_context.template,
                ctx.source_context.offset,
                &usage,
            ) {
                mappings.push(VizeMapping {
                    gen_range: literal_range,
                    src_range,
                    sub_spans: Vec::new(),
                });
            }
        } else {
            ts.push_str("{\n");
            for entry in &entries {
                let prop = entry.prop;
                if entry.inline_callback {
                    append!(
                        *ts,
                        "{call_indent}  // @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n"
                    );
                } else {
                    // This literal infers emit arguments. It is also checked
                    // against the public props input, which widens a required
                    // prop to `T | undefined`. The canonical prop call owns the
                    // authored diagnostic (#10397).
                    append!(
                        *ts,
                        "{call_indent}  // @ts-ignore Inference-only prop; the canonical component-props call owns diagnostics.\n"
                    );
                }
                append!(*ts, "{call_indent}  ");
                let entry_start = ts.len();
                append!(*ts, "\"{}\"", entry.name);
                let key_end = ts.len();
                ts.push_str(": ");
                let value_start = ts.len();
                ts.push_str(entry.value());
                let entry_end = if entry.generated.omits_tail() {
                    entry.generated.mapped_limit(value_start)
                } else {
                    ts.len()
                };
                ts.push_str(",\n");
                mappings.push(VizeMapping {
                    gen_range: entry_start..entry_end,
                    src_range: (ctx.source_context.offset + prop.start) as usize
                        ..(ctx.source_context.offset + prop.end) as usize,
                    sub_spans: prop_entry_sub_spans_with_synthetic_value_gaps(
                        ctx.source_context,
                        prop,
                        entry_start..key_end,
                        value_start..entry_end,
                        entry
                            .owned
                            .as_ref()
                            .map_or(&[], |rewrite| rewrite.synthetic_ranges.as_slice()),
                    ),
                });
                ctx.source_context.inference_only().record_prop_projection(
                    &usage,
                    prop,
                    entry_start..key_end,
                    value_start..entry_end,
                );
                if !ctx.template_binding_access.is_empty()
                    && prop.is_dynamic
                    && !entry.inline_callback
                    && let Some(value) = prop.value.as_ref()
                    && let Some(source) = prop_value_source_range(ctx.source_context, prop)
                {
                    map_rewritten_template_binding(
                        ts,
                        mappings,
                        value_start,
                        source.start,
                        value.as_str(),
                        ctx.template_binding_access,
                    );
                }
            }
            append!(*ts, "{call_indent}}});\n");
        }
    }
    if first_listener && guard.is_some() {
        append!(
            *ts,
            "{call_indent}return undefined as never;\n{}}})();\n",
            ctx.indent
        );
    }
    let prop_key = ctx.prop_key;
    // A loose static emit has no declared runtime arity. Preserve the normal
    // listener-prop fallback; dynamic targets keep their contextual marker.
    let loose_args = if dynamic_component || !ctx.contextual_listener {
        "__L"
    } else {
        "unknown[]"
    };
    append!(
        *ts,
        "{indent}type {inferred_args} = {library} extends true ? (typeof {emit_props} extends {{ __ctx?: infer __X }} ? NonNullable<__X> extends {{ props: infer __P }} ? __P : {{}} : {{}}) extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[] : typeof {emit_props} extends {{ readonly __vizeLooseListener: infer __L extends unknown[] }} ? {loose_args} : typeof {emit_props} extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[];\n",
    );
    let contextless_listener = (ctx.contextual_listener && dynamic_component).then(|| {
        let contextless_listener = cstr!(
        "__{}_{}_{}_contextless_listener",
        ctx.component_type_name,
        scope_id,
        ctx.safe_event_name
    );
    append!(
        *ts,
        "{indent}type {contextless_listener} = {library} extends true ? false : typeof {emit_props} extends {{ readonly __vizeLooseListener: unknown[] }} ? true : false;\n"
    );
        contextless_listener
    });
    Some(InferredEmitArgs {
        arguments: inferred_args,
        contextless_listener,
    })
}

/// One static prop of the inference literal, with the text it emits.
struct InferenceEntry<'u> {
    prop: &'u PassedProp,
    generated: PropSpan<'u>,
    name: String,
    inline_callback: bool,
    owned: Option<CallbackParameterRewrite>,
}

impl InferenceEntry<'_> {
    fn value(&self) -> &str {
        if let Some(owned) = self.owned.as_ref() {
            owned.text.as_str()
        } else if self.inline_callback {
            self.generated.text.trim_start()
        } else {
            self.generated.text.as_str()
        }
    }
}

fn inference_entries<'u>(
    usage: &'u ComponentUsage,
    template_binding_access: &TemplateBindingAccess<'_>,
) -> Vec<InferenceEntry<'u>> {
    // A static `class` next to `:class` is one Vue prop; a second key in
    // this literal would be TS1117 on authored markup that vue-tsc accepts.
    let mut emitted = FxHashSet::default();
    let mut entries = Vec::new();
    for prop in &usage.props {
        if prop.name_is_dynamic || prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        let Some(generated) = generated_prop_span(prop, template_binding_access) else {
            continue;
        };
        let name = to_camel_case(prop.name.as_str());
        if !emitted.insert(name.clone()) {
            continue;
        }
        let inline_callback = is_inline_callback_prop(prop);
        let owned = (inline_callback && !generated.omits_tail())
            .then(|| callback_parameters_on_ignored_line_with_ranges(generated.text.trim_start()))
            .flatten();
        entries.push(InferenceEntry {
            prop,
            generated,
            name,
            inline_callback,
            owned,
        });
    }
    entries
}

pub(super) fn find_component_usage_for_event(
    summary: &Croquis,
    component_name: &str,
    data: &EventHandlerScopeData,
    scope: &Scope,
) -> Option<(usize, ComponentUsage)> {
    vize_croquis::facts::component_usage_slice(summary)
        .iter()
        .enumerate()
        .find(|(_, usage)| {
            usage.name.as_str() == component_name
                && usage.events.iter().any(|event| {
                    !event.name_is_dynamic
                        && event.name.as_str() == data.event_name.as_str()
                        && event_matches_scope(usage, event.start, event.end, data, scope)
                })
        })
        .map(|(index, usage)| (index, usage.clone()))
}

fn event_matches_scope(
    usage: &ComponentUsage,
    event_start: u32,
    event_end: u32,
    data: &EventHandlerScopeData,
    scope: &Scope,
) -> bool {
    let exact = event_start == scope.span.start && event_end == scope.span.end;
    let event_contains_scope = event_start <= scope.span.start && scope.span.end <= event_end;
    let scope_in_usage = usage.start <= scope.span.start && scope.span.end <= usage.end;
    let same_handler = usage.events.iter().any(|event| {
        event.start == event_start
            && event.end == event_end
            && data.handler_expression.as_deref() == event.handler.as_deref()
    });
    exact || event_contains_scope || (scope_in_usage && same_handler)
}
