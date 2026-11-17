use crate::support::fix_history_runner::check_pack;

#[test]
fn direct_inline_callback_props_are_distinguished_from_nested_callbacks() {
    check_pack(
        "direct-callback-props-3546",
        "fix_history_cases::direct_callback_props_3546::direct_inline_callback_props_are_distinguished_from_nested_callbacks",
        "2b82fe00f6a94728c71abe394ab87b00d1704df0",
        Some(3546),
        &[
            "nested-type-arrow-callback",
            "nested-type-arrow-callback-wrong-member",
            "regex-default-callbacks",
            "regex-default-callbacks-wrong-member",
            "array-prop-nested-map-callback",
            "array-prop-nested-map-callback-wrong-element",
        ],
    );
}
