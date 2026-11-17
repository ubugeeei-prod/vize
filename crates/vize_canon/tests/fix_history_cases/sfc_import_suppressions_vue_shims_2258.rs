use crate::support::fix_history_runner::check_pack;

#[test]
fn hoisted_imports_keep_suppressions_and_bare_vue_imports_use_shims() {
    check_pack(
        "sfc-import-suppressions-vue-shims-2258",
        "fix_history_cases::sfc_import_suppressions_vue_shims_2258::hoisted_imports_keep_suppressions_and_bare_vue_imports_use_shims",
        "da3b5e70aeb728b4f5a063354c6284c7e9970eb7",
        Some(2258),
        &[
            "suppressed-import-and-bare-package-vue-shim",
            "unsuppressed-unresolved-import",
            "bare-package-vue-import-uses-shim-type",
        ],
    );
}
