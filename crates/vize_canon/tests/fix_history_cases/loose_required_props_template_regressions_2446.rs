use crate::support::fix_history_runner::check_pack;

#[test]
fn loose_required_props_vfor_unions_and_kebab_attrs() {
    check_pack(
        "loose-required-props-template-regressions-2446",
        "fix_history_cases::loose_required_props_template_regressions_2446::loose_required_props_vfor_unions_and_kebab_attrs",
        "a6c98e7d24eb7323d7f3b9d2543e3e9f2f7def5b",
        Some(2446),
        &[
            "with-defaults-preserves-loose-required-props",
            "with-defaults-model-value-type-still-checked",
            "v-for-infers-items-from-union-of-arrays",
            "v-for-union-item-non-shared-member",
            "define-props-assignable-to-record-helper",
            "define-props-rejected-by-string-record-helper",
            "extended-interface-props-in-template",
            "extended-interface-prop-wrong-member",
            "kebab-prop-with-fallthrough-class-style",
            "kebab-prop-static-value-wrong-type",
        ],
    );
}
