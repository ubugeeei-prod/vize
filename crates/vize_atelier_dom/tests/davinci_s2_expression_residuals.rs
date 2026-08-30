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
    (
        "line_comment_bind_value",
        r#"<div :id="ok // comment"></div>"#,
    ),
    (
        "line_comment_if_condition",
        r#"<div v-if="ok // comment">yes</div>"#,
    ),
    (
        "line_comment_element_interpolation",
        r#"<div>{{ ok // comment }}</div>"#,
    ),
    (
        "line_comment_slot_interpolation",
        r#"<Foo><template #default>{{ ok // comment }}</template></Foo>"#,
    ),
    (
        "line_comment_slot_fallback_interpolation",
        r#"<slot>{{ ok // comment }}</slot>"#,
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
