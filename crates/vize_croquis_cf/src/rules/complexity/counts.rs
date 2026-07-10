use crate::analyzer::CrossFileResult;

pub(super) fn add_count(total: &mut usize, count: usize) {
    *total = total.saturating_add(count);
}

pub(super) fn fallthrough_risk_count(result: &CrossFileResult) -> usize {
    if let Some(summary) = result.fallthrough_summary {
        return summary
            .components_with_potential_issues
            .saturating_add(summary.risky_unconsumed_fallthrough_attr_count);
    }

    result
        .fallthrough_info
        .iter()
        .filter(|info| info.has_potential_issues())
        .count()
}

pub(super) fn logical_operator_count(content: &str) -> usize {
    content
        .matches("&&")
        .count()
        .saturating_add(content.matches("||").count())
}
