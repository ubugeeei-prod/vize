use crate::support::fix_history_runner::check_pack;

#[test]
fn tsx_catch_all_emits_reserved_props_and_union_components() {
    check_pack(
        "tsx-sfc-component-props-5962",
        "fix_history_cases::tsx_sfc_component_props_5962::tsx_catch_all_emits_reserved_props_and_union_components",
        "788000753c5c6bab73faedebc979a18d6db735e2",
        Some(5962),
        &[
            "tsx-catch-all-emit-listener-props",
            "tsx-catch-all-emits-wrong-label-type",
            "tsx-reserved-key-and-ref-on-sfc",
            "tsx-reserved-key-and-ref-wrong-label-type",
            "tsx-union-component-prop-keys",
            "tsx-union-component-wrong-alpha-type",
        ],
    );
}
