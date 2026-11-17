use crate::support::fix_history_runner::check_pack;

#[test]
fn ambient_setup_global_types_preserve_batch_diagnostics() {
    check_pack(
        "setup-ambient-global-types-6211",
        "fix_history_cases::setup_ambient_global_types_6211::ambient_setup_global_types_preserve_batch_diagnostics",
        "a7fcfb4a5a17a4db0cd5fc21326205da7d020e1f",
        Some(6211),
        &[
            "standard-library-types",
            "project-ambient-namespace",
            "missing-type",
            "generic-arity",
            "html-element-property",
            "const-assignment",
            "expect-error-missing-type",
            "unused-expect-error",
        ],
    );
}
