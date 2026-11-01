//! Supplemental known scalar checks for the selected legacy external route.
//! Ordinary eligibility, callback emission, and callback owners stay intact.
use vize_carton::{FxHashMap, FxHashSet, String, append, camelize, capitalize, cstr};
use vize_croquis::{ScopeKind, analysis::ComponentUsage};

use super::super::context::ComponentPropsContext;
use super::super::inline_callback_classifier::LegacyInlineCallbacks;
use super::bindings::{component_name_matches_external_template_binding, has_script_binding};
use crate::virtual_ts::component_reference::component_binding_reference;
use crate::virtual_ts::expressions::{ComponentPropCheckContext, generate_component_prop_checks};
use crate::virtual_ts::helpers::to_camel_case;
use crate::virtual_ts::script_facts;
use crate::virtual_ts::types::VizeMapping;

pub(super) struct ScalarUsages {
    pub(super) rows: Vec<(usize, ComponentUsage)>,
    pub(super) callbacks: FxHashMap<u32, FxHashSet<usize>>,
}

pub(super) fn collect(
    ctx: &ComponentPropsContext<'_, '_>,
    ordinary: &[(usize, &ComponentUsage)],
) -> Option<ScalarUsages> {
    if !ctx.legacy_vue2 || ctx.options.external_template_bindings.is_empty() {
        return None;
    }
    let external = ctx
        .options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect::<FxHashSet<_>>();
    let already_checked = ordinary
        .iter()
        .map(|(_, usage)| usage.start)
        .collect::<FxHashSet<_>>();
    let functions = std::cell::OnceCell::new();
    let mut local_names = FxHashMap::default();
    let mut result = ScalarUsages {
        rows: Vec::new(),
        callbacks: FxHashMap::default(),
    };
    script_facts::with_bindings(ctx.summary, |bindings| {
        for (idx, usage) in vize_croquis::facts::component_usage_list(ctx.summary)
            .into_iter()
            .enumerate()
        {
            if already_checked.contains(&usage.start)
                || has_script_binding(bindings, &usage)
                || !component_name_matches_external_template_binding(usage.name.as_str(), &external)
                || super::named_dynamic_reference(ctx, &usage).is_some()
                || usage.props.len() < 2
            {
                continue;
            }
            let callbacks = usage
                .props
                .iter()
                .enumerate()
                .filter_map(|(index, prop)| {
                    (prop.is_dynamic
                        && prop.value.as_ref().is_some_and(|value| {
                            functions
                                .get_or_init(|| {
                                    LegacyInlineCallbacks::from_root(ctx.template_ast, true)
                                })
                                .has_inline_function(value.as_str())
                        }))
                    .then_some(index)
                })
                .collect::<FxHashSet<_>>();
            if callbacks.is_empty()
                || !usage.props.iter().enumerate().any(|(index, prop)| {
                    !prop.name_is_dynamic
                        && !matches!(prop.name.as_str(), "key" | "ref")
                        && prop.value.is_some()
                        && !callbacks.contains(&index)
                })
            {
                continue;
            }
            // Component references resolve outside replayed template closures.
            // Refuse a template-local name instead of treating the registered
            // ambient spelling as authority. Cache the canonical visibility
            // walk per semantic scope, not once per component/prop.
            if ctx.summary.scopes.get_scope(usage.scope_id).is_none() {
                continue;
            }
            let locals = local_names
                .entry(usage.scope_id.as_u32())
                .or_insert_with(|| {
                    ctx.summary
                        .scopes
                        .bindings_visible_from(usage.scope_id)
                        .into_iter()
                        .filter(|(_, _, kind)| {
                            matches!(
                                kind,
                                ScopeKind::VFor
                                    | ScopeKind::VSlot
                                    | ScopeKind::VMatch
                                    | ScopeKind::VWhen
                            )
                        })
                        .map(|(name, _, _)| name)
                        .collect::<FxHashSet<_>>()
                });
            let root_name = usage.name.as_str().split('.').next().unwrap_or_default();
            let camel = camelize(root_name);
            let pascal = capitalize(camel.as_str());
            if [root_name, camel.as_str(), pascal.as_str()]
                .into_iter()
                .any(|name| locals.contains(name) || bindings.contains_binding(name))
            {
                continue;
            }
            result.callbacks.insert(usage.start, callbacks);
            result.rows.push((idx, usage));
        }
    });
    (!result.rows.is_empty()).then_some(result)
}

pub(super) fn append_aliases(
    ts: &mut String,
    ctx: &ComponentPropsContext<'_, '_>,
    idx: usize,
    usage: &ComponentUsage,
    callbacks: &FxHashSet<usize>,
) {
    let reference = ctx.explicit_generics.usage_reference(
        usage.start,
        component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        ),
    );
    // This supplemental validation view uses only the declared public props.
    // Intrinsic operators avoid capturing authored Record/Omit/PropertyKey.
    // The generic C distributes constructor/callable unions; opaque arms stay
    // unknown. No callback/body type or public component declaration changes.
    append!(
        *ts,
        "  type __vize_legacy_props_{idx}<__C = typeof {reference}> = __C extends abstract new (...__args: infer _Args) => infer __I ? __I extends {{ $props: infer __P }} ? __P : unknown : __C extends (props: infer __P, ...__args: infer _Rest) => unknown ? __P : unknown;\n"
    );
    for (index, prop) in usage.props.iter().enumerate() {
        if prop.name_is_dynamic
            || matches!(prop.name.as_str(), "key" | "ref")
            || prop.value.is_none()
            || callbacks.contains(&index)
        {
            continue;
        }
        // A typed JSON string value has an infallible Display serializer.
        // Keep complete escaping without a panic or an error fallback.
        let raw = cstr!("{}", serde_json::Value::from(prop.name.as_str()));
        let camel = cstr!(
            "{}",
            serde_json::Value::from(to_camel_case(prop.name.as_str()).as_str())
        );
        // Match the existing raw-before-camel value lookup and unknown
        // fallback without widening the component's actual public props.
        append!(
            *ts,
            "  type __vize_legacy_prop_{idx}_{index}<__P = __vize_legacy_props_{idx}, __R = __P extends unknown ? {raw} extends keyof __P ? __P[{raw}] : never : never, __C = __P extends unknown ? {camel} extends keyof __P ? __P[{camel}] : never : never> = [__R] extends [never] ? [__C] extends [never] ? unknown : __C : __R;\n"
        );
    }
}

pub(super) fn generate_root_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_, '_>,
    idx: usize,
    usage: &ComponentUsage,
    callbacks: &FxHashSet<usize>,
) {
    let reference = ctx.explicit_generics.usage_reference(
        usage.start,
        component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        ),
    );
    let mut check_context = ComponentPropCheckContext::new(
        ts,
        mappings,
        ctx.template_binding_access,
        ctx.source_context(),
        "  ",
    )
    .hoist_shared_preamble(ctx.hoist_shared_preamble);
    check_context.legacy_scalar_callback_props = Some(callbacks);
    generate_component_prop_checks(&mut check_context, usage, idx, reference.as_str());
}
