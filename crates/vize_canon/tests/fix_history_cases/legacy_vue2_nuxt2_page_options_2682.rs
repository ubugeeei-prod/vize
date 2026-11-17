use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_accepts_nuxt2_layout_and_head_this_bindings() {
    check_pack(
        "legacy-vue2-nuxt2-page-options-2682",
        "fix_history_cases::legacy_vue2_nuxt2_page_options_2682::legacy_vue2_accepts_nuxt2_layout_and_head_this_bindings",
        "da7b5364bbe88ae02412e5dea627f9ca69804a4e",
        Some(2682),
        &[
            "error-layout-head-this-js",
            "error-layout-head-this-ts",
            "error-layout-async-data-context-wrong-type",
        ],
    );
}
