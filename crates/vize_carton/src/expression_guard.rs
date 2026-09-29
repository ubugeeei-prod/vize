//! Expression nesting guard: whether text is safe to hand to OXC's recursive parser.
//!
//! Lives in `vize_carton` so armature, transform, and codegen entry points all
//! share one guard through the existing re-export shim.

mod analyze;
use analyze::analyze_expression_nesting;

mod operators;
pub mod scan;

use scan::{
    SpeculativeTypeAngleOpen, keyword_allows_regex_after, skip_block_comment, skip_identifier,
    skip_line_comment, skip_number, skip_quoted, skip_regex, skip_template_text,
    speculative_arrow_default_paren, speculative_type_angle_open_kind,
    starts_valid_identifier_escape,
};

pub use scan::is_expression_trailing_trivia;

/// Maximum expression nesting depth accepted before parsing.
///
/// OXC recurses for nested brackets; stack overflow (#956) and a parser timeout
/// at depth 32 (#2944) cannot be caught, so every entry point shares this guard.
pub const MAX_EXPRESSION_NESTING_DEPTH: usize = 31;

/// Per-operator branch depth counted by the cumulative speculative type-angle budget.
/// Logical/nullish operators stop one parser branch, so low-depth comparison chains stay accepted;
/// repeated failed medium-depth type-argument attempts still count toward #4618.
const CUMULATIVE_SPECULATIVE_TYPE_ANGLE_MIN_DEPTH: usize = MAX_EXPRESSION_NESTING_DEPTH / 2;
const MAX_NUMERIC_TOKEN_BYTES: usize = 4096;
/// Product of per-frame `<` counts across nested `<(ident =` defaults.
///
/// One frame stays linear (a flat `x < y ||` tail). Two or more frames that
/// each hold a `<` run multiply: oxc 0.142.0 spent ~134ms at product 972,
/// ~3.6s at 11664, and ran out of memory on the #7116 input (~169000).
/// `8 × 31²` sits between those measurements.
const MAX_ARROW_DEFAULT_ANGLE_PRODUCT: usize =
    8 * MAX_EXPRESSION_NESTING_DEPTH * MAX_EXPRESSION_NESTING_DEPTH;

struct ExpressionNestingAnalysis {
    max_depth: usize,
    delimiters_balanced: bool,
    cumulative_speculative_type_angle_depth: usize,
    excessive_speculative_type_angle_opens: bool,
    excessive_arrow_default_speculation: bool,
    oversized_numeric_token: bool,
}

fn flush_speculative_type_angle_segment(cumulative: &mut usize, segment_depth: &mut usize) {
    if *segment_depth >= CUMULATIVE_SPECULATIVE_TYPE_ANGLE_MIN_DEPTH {
        *cumulative += *segment_depth;
    }
    *segment_depth = 0;
}

/// `<(ident =` frames multiply OXC's speculative reparse. A single frame, or a
/// nest whose other frames hold fewer than two `<`, stays on the linear path.
fn arrow_default_angle_product_exceeds(frames: &[(usize, usize)]) -> bool {
    if frames.iter().filter(|&&(_, count)| count >= 2).count() < 2 {
        return false;
    }
    let mut product = 1usize;
    for &(_, count) in frames {
        product = product.saturating_mul(count.max(1));
        if product > MAX_ARROW_DEFAULT_ANGLE_PRODUCT {
            return true;
        }
    }
    false
}

pub fn expression_nesting_depth(content: &str) -> usize {
    analyze_expression_nesting(content).max_depth
}

/// Returns whether parentheses, brackets, and braces are correctly paired.
pub fn expression_has_balanced_delimiters(content: &str) -> bool {
    analyze_expression_nesting(content).delimiters_balanced
}

/// Returns whether an expression can be handed to OXC's recursive parser safely.
pub fn expression_is_safe_to_parse(content: &str) -> bool {
    let analysis = analyze_expression_nesting(content);
    analysis.delimiters_balanced
        && analysis.max_depth <= MAX_EXPRESSION_NESTING_DEPTH
        && analysis.cumulative_speculative_type_angle_depth <= MAX_EXPRESSION_NESTING_DEPTH
        && !analysis.excessive_speculative_type_angle_opens
        && !analysis.excessive_arrow_default_speculation
        && !analysis.oversized_numeric_token
        && !operators::has_excessive_prefix_operator_run(content)
}

/// Returns true if `content` exceeds [`MAX_EXPRESSION_NESTING_DEPTH`].
#[inline]
pub fn expression_exceeds_max_depth(content: &str) -> bool {
    expression_nesting_depth(content) > MAX_EXPRESSION_NESTING_DEPTH
}
