use crate::virtual_ts::diagnostic_ownership::InputOwner;
use crate::virtual_ts::expressions::ComponentPropSource;
use vize_carton::{FxHashMap, FxHashSet};
use vize_croquis::{Croquis, TemplateExpressionKind};

use super::component_prop_checker::contains_inline_function_prop_value;
use super::component_props::component_usage_has_checkable_binding;
use super::context::ScopeGenerationOptions;
use super::slot_outlet_props::SlotOutletChecks;

#[derive(Default)]
pub(super) struct ComponentPropExpressionPlan {
    pub(super) skipped: FxHashSet<(u32, u32)>,
    pub(super) owners: FxHashMap<(u32, u32, u32), InputOwner>,
}

pub(super) fn collect_component_prop_expression_ranges(
    summary: &Croquis,
    options: &ScopeGenerationOptions<'_, '_>,
    slot_outlets: &SlotOutletChecks,
) -> ComponentPropExpressionPlan {
    if !options.check_options.check_props {
        return ComponentPropExpressionPlan::default();
    }

    let mut plan = ComponentPropExpressionPlan {
        skipped: slot_outlets.expression_ranges(summary),
        owners: FxHashMap::default(),
    };
    let positions: FxHashSet<_> = if options.diagnostic_ownership.is_some() {
        summary
            .template_expressions
            .iter()
            .map(|expr| {
                (
                    expr.scope_id.as_u32(),
                    expr.start as usize,
                    expr.end as usize,
                )
            })
            .collect()
    } else {
        FxHashSet::default()
    };
    let source = ComponentPropSource::new(
        options.template_ast.map(|root| root.source),
        0,
        &summary.scopes,
    );
    let unproven = if options.diagnostic_ownership.is_some() {
        options
            .template_ast
            .map(crate::virtual_ts::diagnostic_ownership::unproven_prop_ranges)
            .unwrap_or_default()
    } else {
        FxHashSet::default()
    };
    let external_template_bindings: FxHashSet<&str> = options
        .virtual_ts_options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    for usage in vize_croquis::facts::component_usage_list(summary) {
        let has_checkable_binding = component_usage_has_checkable_binding(
            summary,
            &usage,
            &external_template_bindings,
            options.component_binding_check,
            options.legacy_vue2,
        );
        if !has_checkable_binding && !options.legacy_vue2 {
            continue;
        }
        if let Some(collector) = options.diagnostic_ownership {
            refuse_ambiguous_prop_spans(collector, &usage);
        }
        for spread in &usage.spread_props {
            // The generic props call checks the entire bag and owns its precise
            // rewrite mapping. A duplicate bare statement would route the same
            // spread through the older token scanner.
            for expr in &summary.template_expressions {
                if expr.kind == TemplateExpressionKind::VBind
                    && expr.scope_id == usage.scope_id
                    && expr.start >= spread.start
                    && expr.end <= spread.end
                    && expr.content.as_str().trim() == spread.expression.as_str().trim()
                {
                    plan.skipped.insert((expr.start, expr.end));
                }
            }
        }
        for prop in &usage.props {
            if unproven.contains(&(prop.start, prop.end))
                && let Some(collector) = options.diagnostic_ownership
                && let Some(owner) = InputOwner::for_expected_projection(&usage, prop)
            {
                collector.borrow_mut().refuse_owner(owner);
            }
            if options.diagnostic_ownership.is_some()
                && let Some(owner) = InputOwner::for_prop(&usage, prop)
                && let Some(value) = source.value_source_range(prop)
                && positions.contains(&(usage.scope_id.as_u32(), value.start, value.end))
            {
                plan.owners.insert(
                    (
                        usage.scope_id.as_u32(),
                        value.start as u32,
                        value.end as u32,
                    ),
                    owner,
                );
            }
            if prop.name_is_dynamic {
                continue;
            }
            let Some(value) = prop.value.as_ref() else {
                continue;
            };
            if !prop.is_dynamic {
                continue;
            }
            let value = value.as_str().trim();
            if !contains_inline_function_prop_value(value) {
                continue;
            }
            // Checkable components validate inline function props through the
            // prop checker. Legacy Vue 2 library globals are intentionally not
            // prop-checked, so emitting those callbacks standalone would leave
            // parameters uncontextualized and report TS7006 false positives.
            for expr in &summary.template_expressions {
                if expr.kind == TemplateExpressionKind::VBind
                    && expr.scope_id == usage.scope_id
                    && expr.start >= prop.start
                    && expr.end <= prop.end
                    && expr.content.as_str().trim() == value
                {
                    plan.skipped.insert((expr.start, expr.end));
                }
            }
        }
    }
    plan
}

/// One authored v-model can produce both a value and a modifier prop. Sharing
/// their source range does not make them the same semantic diagnostic input.
/// Production rows follow authored order; refuse the whole usage if a caller
/// supplies an ordering for which this linear identity proof cannot hold.
fn refuse_ambiguous_prop_spans(
    collector: &crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector,
    usage: &vize_croquis::croquis::ComponentUsage,
) {
    let mut previous = None;
    for prop in &usage.props {
        let span = (prop.start, prop.end);
        if previous.is_some_and(|previous| span < previous) {
            let mut ledger = collector.borrow_mut();
            for prop in &usage.props {
                if let Some(owner) = InputOwner::for_expected_projection(usage, prop) {
                    ledger.refuse_owner(owner);
                }
            }
            return;
        }
        if previous == Some(span)
            && let Some(owner) = InputOwner::for_expected_projection(usage, prop)
        {
            collector.borrow_mut().refuse_owner(owner);
        }
        previous = Some(span);
    }
}
