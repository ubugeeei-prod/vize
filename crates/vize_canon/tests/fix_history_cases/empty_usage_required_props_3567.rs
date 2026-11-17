use crate::support::fix_history_runner::check_pack;

#[test]
fn empty_component_usages_check_required_props() {
    check_pack(
        "empty-usage-required-props-3567",
        "fix_history_cases::empty_usage_required_props_3567::empty_component_usages_check_required_props",
        "9a2ff60327e092636a4a13443694c032266622d5",
        Some(3567),
        &[
            "required-props-bound",
            "empty-usages-missing-required-props",
        ],
    );
}
