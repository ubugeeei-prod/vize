use crate::support::fix_history_runner::check_pack;

#[test]
fn destructured_composable_refs_unwrap_in_template() {
    check_pack(
        "setup-maybe-ref-template-unwrap",
        "fix_history_cases::setup_maybe_ref_template_unwrap::destructured_composable_refs_unwrap_in_template",
        "d9a57097d5e3faedfe731478a2f214f4bf178373",
        None,
        &[
            "composable-refs-interpolated",
            "composable-refs-number-members",
            "composable-ref-wrong-member",
        ],
    );
}
