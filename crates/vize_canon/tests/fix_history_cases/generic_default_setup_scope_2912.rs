use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_default_does_not_replace_setup_parameter() {
    check_pack(
        "generic-default-setup-scope-2912",
        "fix_history_cases::generic_default_setup_scope_2912::generic_default_does_not_replace_setup_parameter",
        "5afffd0b3cb0dd06d9eb347b57f40dba3d9c8e4e",
        Some(2912),
        &[
            "generic-default-wider-constraint",
            "generic-setup-assignment-error",
        ],
    );
}
