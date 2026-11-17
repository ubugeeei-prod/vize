use crate::support::fix_history_runner::check_pack;

#[test]
fn kebab_component_prop_and_event_helpers_are_sanitized() {
    check_pack(
        "kebab-component-type-helpers-641",
        "fix_history_cases::kebab_component_type_helpers_641::kebab_component_prop_and_event_helpers_are_sanitized",
        "93a5c14e4032bbf361b5b989a497b2b16c13139c",
        Some(641),
        &[
            "kebab-widget-prop-and-update-listener",
            "kebab-widget-historical-unresolved",
            "kebab-widget-prop-type-mismatch",
            "kebab-widget-listener-payload-mismatch",
        ],
    );
}
