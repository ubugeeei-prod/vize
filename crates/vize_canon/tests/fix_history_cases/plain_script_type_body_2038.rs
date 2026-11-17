use crate::support::fix_history_runner::check_pack;

#[test]
fn plain_script_type_bodies_preserve_batch_diagnostics() {
    check_pack(
        "plain-script-type-body-2038",
        "fix_history_cases::plain_script_type_body_2038::plain_script_type_bodies_preserve_batch_diagnostics",
        "fd87bb1274a781dad70c3599df9fd3365a1eb870",
        Some(2038),
        &["exported-type-body", "exported-type-body-literal-error"],
    );
}
