use crate::support::fix_history_runner::check_pack;

#[test]
fn script_setup_enums_resolve_as_template_bindings() {
    check_pack(
        "script-setup-enum-template-binding-6025",
        "fix_history_cases::script_setup_enum_template_binding_6025::script_setup_enums_resolve_as_template_bindings",
        "84b9245085683c9eb12f28afadf1278ee93fa27b",
        Some(6025),
        &[
            "historical-literal-enum-comparison",
            "script-setup-enum-read-in-template",
            "script-setup-enum-missing-member",
            "script-setup-enum-wrong-argument",
        ],
    );
}
