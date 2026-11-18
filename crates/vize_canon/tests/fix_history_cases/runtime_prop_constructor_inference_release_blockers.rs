use crate::support::fix_history_runner::check_pack;

#[test]
fn runtime_prop_type_constructors_infer_values() {
    check_pack(
        "runtime-prop-constructor-inference-release-blockers",
        "fix_history_cases::runtime_prop_constructor_inference_release_blockers::runtime_prop_type_constructors_infer_values",
        "498c5f93e058cfa0050f5e308fda5373b682fbbe",
        None,
        &[
            "runtime-prop-type-array-computed-record-types",
            "runtime-prop-type-array-wrong-element-type",
            "computed-map-values-record-types",
            "computed-map-values-wrong-element-type",
            "custom-class-constructor-prop",
            "custom-class-constructor-prop-wrong-member-type",
            "imported-class-constructor-prop",
        ],
    );
}
