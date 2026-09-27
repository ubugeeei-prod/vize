//! Safety predicate shared by every expression parser entry point.

use super::{
    MAX_EXPRESSION_NESTING_DEPTH, analyze_expression_nesting, operators, scan::skip_identifier,
};

/// Returns whether an expression can be handed to OXC's recursive parser safely.
pub fn expression_is_safe_to_parse(content: &str) -> bool {
    // One ASCII identifier-shaped token cannot contain recursive parser syntax.
    // Keywords still pass through OXC's expression admission rule.
    let bytes = content.as_bytes();
    if matches!(bytes.first(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$'))
        && skip_identifier(bytes, 1) == bytes.len()
    {
        return true;
    }
    let analysis = analyze_expression_nesting(content);
    analysis.delimiters_balanced
        && analysis.max_depth <= MAX_EXPRESSION_NESTING_DEPTH
        && analysis.cumulative_speculative_type_angle_depth <= MAX_EXPRESSION_NESTING_DEPTH
        && !analysis.excessive_speculative_type_angle_opens
        && !analysis.oversized_numeric_token
        && !operators::has_excessive_prefix_operator_run(content)
}
