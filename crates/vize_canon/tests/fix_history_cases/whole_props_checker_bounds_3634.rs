use crate::support::fix_history_runner::check_pack;

#[test]
fn whole_props_checks_target_the_unmodified_child_props() {
    check_pack(
        "whole-props-checker-bounds-3634",
        "fix_history_cases::whole_props_checker_bounds_3634::whole_props_checks_target_the_unmodified_child_props",
        "08297d4ffd2d00cf0dc8e729665dfc430e8cdf01",
        Some(3634),
        &[
            "string-literal-union-prop-valid",
            "string-literal-union-prop-wrong-literal",
            "wide-props-complete-usage",
            "wide-props-missing-required-title",
            "attribute-only-required-props-edges",
            "generic-emit-guard-missing-count",
            "generic-emit-guard-count-bound",
        ],
    );
}
