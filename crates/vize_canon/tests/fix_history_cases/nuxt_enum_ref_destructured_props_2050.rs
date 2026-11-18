use crate::support::fix_history_runner::check_pack;

#[test]
fn enum_ref_comparisons_and_renamed_rest_props_have_no_false_positives() {
    check_pack(
        "nuxt-enum-ref-destructured-props-2050",
        "fix_history_cases::nuxt_enum_ref_destructured_props_2050::enum_ref_comparisons_and_renamed_rest_props_have_no_false_positives",
        "edac70a4b8aaf6dfedbd53d0fbe73149b9a79bed",
        Some(2050),
        &[
            "enum-ref-template-comparisons",
            "enum-ref-template-string-comparison",
            "typed-route-destructured-rest-props",
            "renamed-destructured-prop-wrong-type",
        ],
    );
}
