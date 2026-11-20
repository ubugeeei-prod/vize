use crate::support::fix_history_runner::check_pack;

#[test]
fn template_dollar_globals_resolve_through_component_custom_properties() {
    check_pack(
        "template-dollar-globals-custom-properties-849",
        "fix_history_cases::template_dollar_globals_custom_properties_849::template_dollar_globals_resolve_through_component_custom_properties",
        "822768792a095efe41e240a4a86669ac73b1b8c5",
        Some(849),
        &[
            "dollar-global-from-custom-properties",
            "dollar-global-argument-type-checked",
            "dollar-global-beside-missing-identifier",
        ],
    );
}
