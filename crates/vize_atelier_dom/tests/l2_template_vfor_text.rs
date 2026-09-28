//! Template v-for fragments need text vnodes for interpolation children.

#[expect(
    dead_code,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; each binary uses a subset"
)]
mod support;

#[test]
fn template_v_for_text_children_match_the_shipped_lane() {
    support::assert_l2_matches_shipped(&[
        (
            "interpolation_then_element",
            r#"<template v-for="item in items" :key="item">{{ item }}<i>/</i></template>"#,
        ),
        (
            "element_then_interpolation",
            r#"<template v-for="item in items"><i>/</i>{{ item }}</template>"#,
        ),
        (
            "interpolation_then_text",
            r#"<template v-for="item in items">{{ item }} text</template>"#,
        ),
    ]);
}
