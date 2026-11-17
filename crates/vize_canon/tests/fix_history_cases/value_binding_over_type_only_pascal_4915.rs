use crate::support::fix_history_runner::check_pack;

#[test]
fn value_bindings_win_over_type_only_pascal_component_aliases() {
    check_pack(
        "value-binding-over-type-only-pascal-4915",
        "fix_history_cases::value_binding_over_type_only_pascal_4915::value_bindings_win_over_type_only_pascal_component_aliases",
        "ad99a95135db04ceff3a0d882fefbe5162fddb05",
        Some(4915),
        &[
            "camel-value-binding-beside-type-only-pascal-import",
            "camel-value-binding-component-props",
            "camel-value-binding-component-wrong-prop",
        ],
    );
}
