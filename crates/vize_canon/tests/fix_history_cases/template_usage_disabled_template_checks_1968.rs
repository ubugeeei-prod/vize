use crate::support::fix_history_runner::check_pack;

#[test]
fn disabled_template_checks_still_mark_template_bindings_used() {
    check_pack(
        "template-usage-disabled-template-checks-1968",
        "fix_history_cases::template_usage_disabled_template_checks_1968::disabled_template_checks_still_mark_template_bindings_used",
        "99bc5034f1671bef7de6c83047cd9c264d05562f",
        Some(1968),
        &[
            "template-bindings-used-script-local-unused",
            "template-bindings-used-no-unused-locals",
            "binding-dropped-from-template-is-unused",
        ],
    );
}
