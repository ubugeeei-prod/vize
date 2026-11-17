use crate::support::fix_history_runner::check_pack;

#[test]
fn template_ref_unwrap_follows_installed_vue_ref_value() {
    check_pack(
        "template-ref-unwrap-installed-vue-1814",
        "fix_history_cases::template_ref_unwrap_installed_vue_1814::template_ref_unwrap_follows_installed_vue_ref_value",
        "5bcfe78a5be8845d60d43a3b63eb0aa5111b3d7f",
        Some(1814),
        &[
            "historical-props-and-ref",
            "historical-ref-only",
            "unwrapped-ref-number-member",
            "unwrapped-ref-has-no-value-member",
        ],
    );
}
