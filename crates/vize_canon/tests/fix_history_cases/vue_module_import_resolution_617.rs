use crate::support::fix_history_runner::check_pack;

#[test]
fn vue_module_imports_resolve_without_wildcard_stub() {
    check_pack(
        "vue-module-import-resolution-617",
        "fix_history_cases::vue_module_import_resolution_617::vue_module_imports_resolve_without_wildcard_stub",
        "2e884f7b2df816c675f1473bba681b4429a4f458",
        Some(617),
        &[
            "ts-entry-imports-existing-vue",
            "ts-entry-imports-missing-vue",
            "async-setup-invocation",
            "async-setup-body-mismatch",
        ],
    );
}
