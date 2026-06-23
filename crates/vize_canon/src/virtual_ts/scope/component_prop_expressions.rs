use vize_carton::FxHashSet;
use vize_croquis::{Croquis, TemplateExpressionKind};

use super::component_prop_checker::is_inline_function_prop_value;
use super::component_props::component_usage_has_checkable_binding;
use super::context::ScopeGenerationOptions;
use crate::virtual_ts::types::VirtualTsOptions;

pub(super) fn collect_component_prop_expression_ranges(
    summary: &Croquis,
    virtual_ts_options: &VirtualTsOptions,
    options: &ScopeGenerationOptions<'_>,
) -> FxHashSet<(u32, u32)> {
    if !options.check_options.check_props {
        return FxHashSet::default();
    }

    let external_template_bindings: FxHashSet<&str> = virtual_ts_options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    let mut ranges = FxHashSet::default();
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
        for prop in &usage.props {
            let Some(value) = prop.value.as_ref() else {
                continue;
            };
            if !prop.is_dynamic {
                continue;
            }
            let value = value.as_str().trim();
            if !is_inline_function_prop_value(value) {
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
