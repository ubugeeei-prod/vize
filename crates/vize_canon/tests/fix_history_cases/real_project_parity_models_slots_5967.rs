use crate::support::fix_history_runner::check_pack;

#[test]
fn real_project_parity_models_events_slot_spreads_and_aliases() {
    check_pack(
        "real-project-parity-models-slots-5967",
        "fix_history_cases::real_project_parity_models_slots_5967::real_project_parity_models_events_slot_spreads_and_aliases",
        "4b889011a1f6cdea06aac9ef1779a048d2518b74",
        Some(5967),
        &[
            "runtime-model-defaulted-string",
            "runtime-model-optional-keeps-undefined",
            "runtime-model-optional-read-as-string",
            "model-get-set-distinct-read-write",
            "model-get-set-wrong-write-type",
            "model-get-set-contextual-callbacks",
            "model-get-set-contextual-wrong-member",
            "unresolved-component-event-callback-any",
            "resolved-component-event-bad-handler",
            "resolved-component-event-matching-handler",
            "dynamic-slot-spread-unknown-payload",
            "generic-slot-spread-exact-payload",
            "slot-spread-explicit-prop-still-checked",
            "slot-spread-explicit-prop-matching",
            "unconfigured-alias-vue-type-import-stays-authored",
            "configured-alias-vue-type-import",
            "configured-alias-vue-type-import-wrong-item",
            "escaped-model-and-prop-names",
            "escaped-model-name-read-as-number",
            "imported-type-only-props-with-models",
            "imported-generic-props-instantiation-wrong-item",
        ],
    );
}
