use crate::support::fix_history_runner::check_pack;

#[test]
fn runtime_prop_helper_emits_valid_typescript() {
    check_pack(
        "runtime-prop-helper-syntax",
        "fix_history_cases::runtime_prop_helper_syntax::runtime_prop_helper_emits_valid_typescript",
        "e49925f0fbde257854af2aa73e34b45be7f14b3d",
        None,
        &[
            "runtime-props-parse-and-check",
            "runtime-string-prop-type-error",
        ],
    );
}
