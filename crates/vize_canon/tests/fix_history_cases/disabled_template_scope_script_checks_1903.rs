use crate::support::fix_history_runner::check_pack;

#[test]
fn disabled_template_bindings_skip_template_but_check_script() {
    check_pack(
        "disabled-template-scope-script-checks-1903",
        "fix_history_cases::disabled_template_scope_script_checks_1903::disabled_template_bindings_skip_template_but_check_script",
        "dfdce7dd95f906520de680087343f04872f4945c",
        Some(1903),
        &[
            "template-skipped-script-clean",
            "template-skipped-script-error-kept",
        ],
    );
}
