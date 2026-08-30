//! P2-11 corpus residual witnesses for expression edge forms.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("reserved_bind_value", r#"<div :class="class"></div>"#),
    (
        "trailing_comment_bind_value",
        r#"<div :child="processDateItem(child)/* deprecated, use date instead */"></div>"#,
    ),
    ("empty_root_interpolation", "{{  }}"),
    (
        "empty_slot_interpolation",
        "<Foo><template #more-actions>{{  }}</template></Foo>",
    ),
];

#[test]
fn s2_expression_residuals_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
