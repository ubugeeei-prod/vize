use crate::support::fix_history_runner::check_pack;

#[test]
fn dynamic_is_on_v_for_elements_reads_aliases_through_the_strict_context() {
    check_pack(
        "dynamic-component-v-for-scope-strict",
        "fix_history_cases::dynamic_component_v_for_scope_strict::dynamic_is_on_v_for_elements_reads_aliases_through_the_strict_context",
        "7f802b69bf57fc555e65174d33d8319fc4753aa4",
        None,
        &[
            "is-before-for",
            "for-before-is",
            "is-before-for-property-error",
            "is-before-for-unknown-name",
            "for-before-is-unknown-name",
        ],
    );
}
