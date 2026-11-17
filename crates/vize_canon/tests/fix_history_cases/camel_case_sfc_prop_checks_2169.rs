use crate::support::fix_history_runner::check_pack;

#[test]
fn camel_case_sfc_child_props_are_still_checked() {
    check_pack(
        "camel-case-sfc-prop-checks-2169",
        "fix_history_cases::camel_case_sfc_prop_checks_2169::camel_case_sfc_child_props_are_still_checked",
        "fffcb76482df4002955407669003ec45dd0615ba",
        Some(2169),
        &[
            "camel-case-prop-mismatch",
            "kebab-case-prop-mismatch",
            "camel-and-kebab-props-match",
            "forwarded-optional-props",
            "options-api-prop-type-long-comments",
        ],
    );
}
