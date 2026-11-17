use crate::support::fix_history_runner::check_pack;

#[test]
fn options_spread_only_computed_reports_no_synthetic_unused_alias() {
    check_pack(
        "options-spread-only-computed-unused",
        "fix_history_cases::options_spread_only_computed_unused::options_spread_only_computed_reports_no_synthetic_unused_alias",
        "be564bc1b423bcc304e9d11a868467c82f0a526d",
        None,
        &[
            "spread-only-computed-no-unused-alias",
            "spread-only-computed-authored-unused-local",
            "spread-only-computed-wrong-member-use",
        ],
    );
}
