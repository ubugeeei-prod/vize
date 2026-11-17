use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_preserves_optional_union_props() {
    check_pack(
        "with-defaults-optional-union-2913",
        "fix_history_cases::with_defaults_optional_union_2913::with_defaults_preserves_optional_union_props",
        "fc0e623bb9d2e9d50f496d98ca5f78bf1a607218",
        Some(2913),
        &["defaulted-discriminant-union", "union-prop-ref-type-error"],
    );
}
