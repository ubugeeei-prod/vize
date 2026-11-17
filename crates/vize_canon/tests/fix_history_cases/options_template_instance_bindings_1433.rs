use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_template_bindings_keep_instance_types() {
    check_pack(
        "options-template-instance-bindings-1433",
        "fix_history_cases::options_template_instance_bindings_1433::options_api_template_bindings_keep_instance_types",
        "87701cf384daf9ccb001e3ab5ea19f74e7938397",
        Some(1433),
        &[
            "data-binding-number-method",
            "data-binding-wrong-argument",
            "props-data-computed-methods-instance",
            "method-return-wrong-member",
        ],
    );
}
