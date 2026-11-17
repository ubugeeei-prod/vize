use crate::support::fix_history_runner::check_pack;

#[test]
fn component_static_and_dynamic_class_bindings_merge() {
    check_pack(
        "component-merged-class-bindings-2538",
        "fix_history_cases::component_merged_class_bindings_2538::component_static_and_dynamic_class_bindings_merge",
        "ed38d543eeed916cf98a6be84d58e3b4b84015ce",
        Some(2538),
        &[
            "static-and-dynamic-class-merged",
            "merged-class-sibling-prop-type-error",
        ],
    );
}
