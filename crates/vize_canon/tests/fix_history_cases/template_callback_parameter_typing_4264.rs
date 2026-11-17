use crate::support::fix_history_runner::check_pack;

#[test]
fn template_callback_parameters_use_declared_slot_transition_and_model_types() {
    check_pack(
        "template-callback-parameter-typing-4264",
        "fix_history_cases::template_callback_parameter_typing_4264::template_callback_parameters_use_declared_slot_transition_and_model_types",
        "7e1f9b31b014d60ef91021d26a83fcda6fa661a1",
        Some(4264),
        &[
            "slot-outlet-callbacks-declared-props-clean",
            "slot-outlet-callbacks-declared-props",
            "untyped-slot-outlet-callback-implicit-any",
            "external-member-update-model-handlers",
            "local-sfc-update-model-identifier-handler-strict",
            "transition-group-hook-handlers-elements",
            "transition-camel-case-hook-handlers-elements",
            "dynamic-component-custom-member-handler",
        ],
    );
}
