use crate::support::fix_history_runner::check_pack;

#[test]
fn recovered_malformed_attributes_reach_the_checker() {
    check_pack(
        "template-parser-attribute-recovery-347",
        "fix_history_cases::template_parser_attribute_recovery_347::recovered_malformed_attributes_reach_the_checker",
        "731cce813a41207950923d89099ac2c87047c0b6",
        Some(347),
        &[
            "missing-equals-static-prop-recovered",
            "missing-equals-static-prop-wrong-type",
            "missing-dynamic-argument-end-recovered",
            "missing-dynamic-argument-end-unknown-value",
        ],
    );
}
