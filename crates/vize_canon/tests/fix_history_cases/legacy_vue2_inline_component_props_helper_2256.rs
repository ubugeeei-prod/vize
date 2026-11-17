use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_preamble_defines_component_props_helpers() {
    check_pack(
        "legacy-vue2-inline-component-props-helper-2256",
        "fix_history_cases::legacy_vue2_inline_component_props_helper_2256::legacy_vue2_preamble_defines_component_props_helpers",
        "8fe0c8dc8e1abd3df101d026f0b6efb876ef59d4",
        Some(2256),
        &[
            "options-data-component",
            "setup-props-component-instance",
            "setup-prop-wrong-script-assignment",
        ],
    );
}
