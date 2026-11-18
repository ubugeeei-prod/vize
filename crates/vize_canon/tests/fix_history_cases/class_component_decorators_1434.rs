use crate::support::fix_history_runner::check_pack;

#[test]
fn class_component_default_export_keeps_its_decorators() {
    check_pack(
        "class-component-decorators-1434",
        "fix_history_cases::class_component_decorators_1434::class_component_default_export_keeps_its_decorators",
        "37b62dd501f041f8ea49684c412ac8912fa81b21",
        Some(1434),
        &[
            "decorated-class-default-export",
            "decorated-class-member-wrong-type",
            "decorated-class-template-wrong-argument",
            "sync-fields-and-setters-assignable",
            "sync-field-wrong-type-and-getter-only",
        ],
    );
}
