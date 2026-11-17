use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_identifier_options_props_keep_required_literals() {
    check_pack(
        "legacy-vue2-required-options-props-2004",
        "fix_history_cases::legacy_vue2_required_options_props_2004::legacy_vue2_identifier_options_props_keep_required_literals",
        "7070fe414bbf008b6d72761ccfcba6a977bc1096",
        Some(2004),
        &[
            "required-identifier-props-defined-in-setup",
            "optional-identifier-props-possibly-undefined",
        ],
    );
}
