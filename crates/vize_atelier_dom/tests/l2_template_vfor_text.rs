//! Template v-for fragments need text vnodes for interpolation children.

use crate::support;

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
