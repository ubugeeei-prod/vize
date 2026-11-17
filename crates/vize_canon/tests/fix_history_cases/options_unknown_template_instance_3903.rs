use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_unknown_template_names_resolve_on_the_public_instance() {
    check_pack(
        "options-unknown-template-instance-3903",
        "fix_history_cases::options_unknown_template_instance_3903::options_api_unknown_template_names_resolve_on_the_public_instance",
        "263094d86067ac5149b40d537585b16e6aa24865",
        Some(3903),
        &[
            "options-data-binding",
            "options-unknown-name-on-instance",
            "script-declared-namespace",
            "script-declared-namespace-unknown-name",
            "setup-next-to-plain-script",
            "setup-next-to-plain-script-unknown-name",
        ],
    );
}
