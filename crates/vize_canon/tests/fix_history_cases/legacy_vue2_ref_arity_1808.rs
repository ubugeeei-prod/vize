use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_template_ref_unwrap_accepts_single_parameter_ref() {
    check_pack(
        "legacy-vue2-ref-arity-1808",
        "fix_history_cases::legacy_vue2_ref_arity_1808::legacy_vue2_template_ref_unwrap_accepts_single_parameter_ref",
        "724abf846703cc4461bb10c957ee5b5eefdf58ca",
        Some(1808),
        &[
            "legacy-ref-unwrap",
            "legacy-props-and-ref-unwrap",
            "legacy-ref-unwrap-wrong-member",
            "legacy-prop-wrong-member",
            "authored-two-argument-ref-rejected",
        ],
    );
}
