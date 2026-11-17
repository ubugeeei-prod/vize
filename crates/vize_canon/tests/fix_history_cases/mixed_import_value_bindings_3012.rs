use crate::support::fix_history_runner::check_pack;

#[test]
fn mixed_default_named_and_namespace_imports_keep_value_bindings() {
    check_pack(
        "mixed-import-value-bindings-3012",
        "fix_history_cases::mixed_import_value_bindings_3012::mixed_default_named_and_namespace_imports_keep_value_bindings",
        "8bf88a7a1bf5aca18cde096ea847d186370905df",
        Some(3012),
        &[
            "mixed-default-named-namespace-imports",
            "mixed-default-import-type-still-checked",
        ],
    );
}
