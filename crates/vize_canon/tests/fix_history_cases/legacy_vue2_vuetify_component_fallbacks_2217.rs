use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_keeps_global_component_props_and_events_permissive() {
    check_pack(
        "legacy-vue2-vuetify-component-fallbacks-2217",
        "fix_history_cases::legacy_vue2_vuetify_component_fallbacks_2217::legacy_vue2_keeps_global_component_props_and_events_permissive",
        "aa03206ddd5518831c26bbc62c88b516c592d11d",
        Some(2217),
        &[
            "global-date-picker-props-and-input-event",
            "imported-component-unknown-event-any",
            "unresolved-component-keyed-props-unchecked",
            "imported-component-prop-wrong-type",
        ],
    );
}
