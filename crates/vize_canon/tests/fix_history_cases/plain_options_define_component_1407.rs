use crate::support::fix_history_runner::check_pack;

#[test]
fn plain_options_object_default_export_gets_instance_this() {
    check_pack(
        "plain-options-define-component-1407",
        "fix_history_cases::plain_options_define_component_1407::plain_options_object_default_export_gets_instance_this",
        "df454d1dad18d3534176b6c9e8544fab636a2e88",
        Some(1407),
        &[
            "this-data-access-in-computed",
            "this-data-access-wrong-return-type",
            "dual-block-options-object",
            "dual-block-options-wrong-type",
            "single-line-options-object",
            "single-line-options-wrong-name",
        ],
    );
}
