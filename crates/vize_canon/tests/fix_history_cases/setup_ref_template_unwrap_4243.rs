use crate::support::fix_history_runner::check_pack;

#[test]
fn imported_setup_refs_unwrap_in_template_expressions() {
    check_pack(
        "setup-ref-template-unwrap-4243",
        "fix_history_cases::setup_ref_template_unwrap_4243::imported_setup_refs_unwrap_in_template_expressions",
        "59d893a34c9def4ad6a4cdc46d148f97f3422e9f",
        Some(4243),
        &[
            "imported-refs-clean-template",
            "imported-ref-misspelled-member",
            "imported-ref-template-value-access",
            "plain-value-object-not-unwrapped",
            "script-scope-ref-not-unwrapped",
        ],
    );
}
