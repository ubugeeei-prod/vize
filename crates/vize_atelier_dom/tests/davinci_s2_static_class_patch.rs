//! P2-11 static class literal patch-flag witnesses.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("string", r#"<div :class="'card'"></div>"#),
    ("array", r#"<div :class="['card', `quiet`]"></div>"#),
    (
        "object",
        r#"<div :class="{ card: true, quiet: false }"></div>"#,
    ),
    (
        "text_and_array",
        r#"<p :class="['copy']">{{ message }}</p>"#,
    ),
];

#[test]
fn s2_static_class_literal_patch_flags_match_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
