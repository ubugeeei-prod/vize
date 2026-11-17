use crate::support::fix_history_runner::check_pack;

#[test]
fn fallthrough_only_usages_keep_required_props() {
    check_pack(
        "fallthrough-attrs-required-props-3579",
        "fix_history_cases::fallthrough_attrs_required_props_3579::fallthrough_only_usages_keep_required_props",
        "1dba759d5b48406481494db09fa152cfcce4a68a",
        Some(3579),
        &[
            "fallthrough-attrs-with-required-props-bound",
            "fallthrough-only-usages-missing-required-props",
            "explicit-class-prop-number",
            "explicit-class-prop-mismatch-once",
        ],
    );
}
