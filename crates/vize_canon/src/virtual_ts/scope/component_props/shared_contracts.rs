//! Reuse ordinary component contract identities within one template scope.
use super::{
    ComponentPropsContext, ComponentUsage, FxHashMap, String, component_binding_reference,
    constructor_instance, named_dynamic_reference, to_safe_identifier_fragment,
    usage_needs_per_prop_aliases,
};

#[derive(Clone, Copy)]
pub(super) struct SharedComponentContract {
    pub(super) canonical_idx: usize,
    pub(super) needs_value_props: bool,
}

pub(super) fn shared_component_contracts(
    ctx: &ComponentPropsContext<'_, '_>,
    usages: &[(usize, &ComponentUsage)],
) -> FxHashMap<usize, SharedComponentContract> {
    let mut groups: FxHashMap<(String, String, bool), SharedComponentContract> =
        FxHashMap::default();
    for &(idx, usage) in usages {
        if constructor_instance::root_slot_scope(
            &ctx.summary.scopes,
            usage,
            ctx.hoist_shared_preamble,
        )
        .is_some()
        {
            continue;
        }
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
        if constructor_instance::root_slot_scope(
            &ctx.summary.scopes,
            usage,
            ctx.hoist_shared_preamble,
        )
        .is_some()
        {
            continue;
        }
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
