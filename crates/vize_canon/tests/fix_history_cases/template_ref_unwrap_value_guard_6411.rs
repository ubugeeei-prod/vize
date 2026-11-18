use crate::support::fix_history_runner::check_pack;

#[test]
fn template_unwrap_keeps_imported_function_signatures_without_vue_types() {
    check_pack(
        "template-ref-unwrap-value-guard-6411",
        "fix_history_cases::template_ref_unwrap_value_guard_6411::template_unwrap_keeps_imported_function_signatures_without_vue_types",
        "816db89fa9a95d479894d690c7faa63f9276c962",
        Some(6411),
        &[
            "vue-types-unavailable-imported-call",
            "vue-types-unavailable-imported-call-wrong-arg",
            "real-vue-ref-unwraps-into-call",
            "real-vue-ref-unwraps-wrong-value",
        ],
    );
}
