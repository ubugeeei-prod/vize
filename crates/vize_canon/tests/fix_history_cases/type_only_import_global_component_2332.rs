use crate::support::fix_history_runner::check_pack;

#[test]
fn type_only_import_does_not_shadow_global_component() {
    check_pack(
        "type-only-import-global-component-2332",
        "fix_history_cases::type_only_import_global_component_2332::type_only_import_does_not_shadow_global_component",
        "695976764435b7d68addf0d440fe79c2a43bdb5e",
        Some(2332),
        &[
            "type-only-import-keeps-global-component",
            "type-only-import-global-component-wrong-prop",
        ],
    );
}
