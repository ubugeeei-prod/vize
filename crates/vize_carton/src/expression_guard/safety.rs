//! Safety predicate shared by every expression parser entry point.

use super::{MAX_EXPRESSION_NESTING_DEPTH, analyze_expression_nesting, operators};

/// Returns whether an expression can be handed to OXC's recursive parser safely.
pub fn expression_is_safe_to_parse(content: &str) -> bool {
    let analysis = analyze_expression_nesting(content);
    analysis.delimiters_balanced
        && analysis.max_depth <= MAX_EXPRESSION_NESTING_DEPTH
        && analysis.cumulative_speculative_type_angle_depth <= MAX_EXPRESSION_NESTING_DEPTH
        && !analysis.excessive_speculative_type_angle_opens
        && !analysis.oversized_numeric_token
        && !operators::has_excessive_prefix_operator_run(content)
}
