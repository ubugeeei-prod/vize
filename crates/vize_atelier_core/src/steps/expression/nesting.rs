//! Re-export shim: the expression nesting guard lives in
//! [`vize_s0::expression_guard`] since Davinci P1-5, so the armature
//! retained-expression parse site shares the exact guard these transform and
//! codegen entry points use. Import paths through this module are preserved.
//! Since P1-9 the transform rewrite runs these scans only on its legacy
//! re-parse chain: an admitted retained AST is proof the same guard passed
//! at the armature parse over the same bytes, so the AST-driven path skips
//! them.

pub use vize_s0::expression_guard::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_exceeds_max_depth, expression_has_balanced_delimiters,
    expression_is_safe_to_parse, expression_nesting_depth,
};

pub(crate) mod scan {
    pub(crate) use vize_s0::expression_guard::scan::{
        keyword_allows_regex_after, skip_identifier, skip_line_comment, skip_number, skip_quoted,
        skip_regex,
    };
}
