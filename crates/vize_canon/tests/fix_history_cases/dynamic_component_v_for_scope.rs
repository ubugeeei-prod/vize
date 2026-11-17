use crate::support::fix_history_runner::check_pack;

#[test]
fn dynamic_is_on_v_for_elements_preserves_batch_diagnostics() {
    check_pack(
        "dynamic-component-v-for-scope",
        "fix_history_cases::dynamic_component_v_for_scope::dynamic_is_on_v_for_elements_preserves_batch_diagnostics",
        "7f802b69bf57fc555e65174d33d8319fc4753aa4",
        None,
        &[
            "is-before-for",
            "for-before-is",
            "is-before-for-property-error",
        ],
    );
}
