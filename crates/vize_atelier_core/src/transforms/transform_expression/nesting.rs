//! Re-export shim: the expression nesting guard lives in
//! [`vize_carton::expression_guard`] since Davinci P1-5, so the armature
//! retained-expression parse site shares the exact guard these transform and
//! codegen entry points use. Import paths through this module are preserved;
//! the P1-9 transform rewrite still owns the call sites around it.

pub use vize_carton::expression_guard::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_exceeds_max_depth, expression_has_balanced_delimiters,
    expression_is_safe_to_parse, expression_nesting_depth,
};

pub(crate) mod scan {
    pub(crate) use vize_carton::expression_guard::scan::{
        keyword_allows_regex_after, skip_identifier, skip_line_comment, skip_number, skip_quoted,
        skip_regex,
    };
}
