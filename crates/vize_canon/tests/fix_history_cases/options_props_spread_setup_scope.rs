use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_spread_runtime_props_stay_in_setup_scope() {
    check_pack(
        "options-props-spread-setup-scope",
        "fix_history_cases::options_props_spread_setup_scope::options_api_spread_runtime_props_stay_in_setup_scope",
        "13ae68d9a09d91ca85353c9b769d4c78e6ed6f18",
        None,
        &[
            "props-spread-with-instance-members",
            "props-spread-this-setup-return-wrong-argument",
            "props-spread-parent-binding",
            "props-spread-parent-wrong-meta",
        ],
    );
}
