use vize_carton::FxHashSet;
use vize_croquis::{Croquis, TemplateExpressionKind};

use super::component_prop_checker::contains_inline_function_prop_value;
use super::component_props::component_usage_has_checkable_binding;
use super::context::ScopeGenerationOptions;
use super::slot_outlet_props::SlotOutletChecks;

pub(super) fn collect_component_prop_expression_ranges(
    summary: &Croquis,
    options: &ScopeGenerationOptions<'_, '_>,
    slot_outlets: &SlotOutletChecks,
) -> FxHashSet<(u32, u32)> {
    if !options.check_options.check_props {
        return FxHashSet::default();
    }

    let mut ranges = slot_outlets.expression_ranges(summary);
    let external_template_bindings: FxHashSet<&str> = options
        .virtual_ts_options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    for usage in &summary.component_usages {
        let has_checkable_binding = component_usage_has_checkable_binding(
            summary,
            usage,
            &external_template_bindings,
            options.check_unresolved_global_components,
            options.legacy_vue2,
        );
        if !has_checkable_binding && !options.legacy_vue2 {
            continue;
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
                    ranges.insert((expr.start, expr.end));
                }
            }
        }
        for prop in &usage.props {
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
                    ranges.insert((expr.start, expr.end));
                }
            }
        }
    }
    ranges
}
