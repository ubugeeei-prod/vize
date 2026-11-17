use crate::support::fix_history_runner::check_pack;

#[test]
fn known_aria_and_custom_data_attrs_fall_through_but_unknown_aria_is_rejected() {
    check_pack(
        "aria-data-fallthrough-attrs-4734",
        "fix_history_cases::aria_data_fallthrough_attrs_4734::known_aria_and_custom_data_attrs_fall_through_but_unknown_aria_is_rejected",
        "8298520356b8ee9d0d3b7ab96e10791f0150652c",
        Some(4734),
        &[
            "mono-root-known-aria-and-data-attrs",
            "mono-root-aria-label",
            "mono-root-unknown-aria-prefix-rejected",
            "mono-root-aria-label-wrong-title",
            "generic-root-custom-data-attrs",
            "generic-root-custom-data-attrs-wrong-title",
        ],
    );
}
