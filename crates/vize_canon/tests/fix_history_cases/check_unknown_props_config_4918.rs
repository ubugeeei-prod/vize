use crate::support::fix_history_runner::check_pack;

#[test]
fn unknown_component_props_follow_vue_compiler_options() {
    check_pack(
        "check-unknown-props-config-4918",
        "fix_history_cases::check_unknown_props_config_4918::unknown_component_props_follow_vue_compiler_options",
        "a58a9ed3a59120922707ac9ec929a8d610ccc531",
        Some(4918),
        &[
            "vue-tsc-default-config-accepts-unknown-extras",
            "explicit-check-unknown-props-false-overrides-strict-templates",
            "inherited-check-unknown-props-reports-unknown-extras",
        ],
    );
}
