use crate::support::fix_history_runner::check_pack;

#[test]
fn destructured_v_for_value_bindings_keep_item_types() {
    check_pack(
        "v-for-destructured-value-bindings",
        "fix_history_cases::v_for_destructured_value_bindings::destructured_v_for_value_bindings_keep_item_types",
        "b10876e9137243be48f9405e44b88003c1906f49",
        None,
        &[
            "object-pattern-v-for",
            "object-pattern-second-binding-string",
            "object-pattern-first-binding-wrong-member",
            "nested-object-pattern-v-for",
            "nested-object-pattern-wrong-member",
        ],
    );
}
