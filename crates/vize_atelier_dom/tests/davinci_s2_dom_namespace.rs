//! P2-11 foreign namespace witness for the S2 DOM lane.
//!
//! SVG / MathML render output must stay byte-for-byte aligned with the shipped
//! lane because Vue's runtime infers namespaces from contiguous vnode trees.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("svg_static_tree", "<svg><path/></svg>"),
    (
        "svg_foreign_object_boundary",
        r#"<svg><foreignObject><div>hi</div></foreignObject><rect x="1" y="1"/></svg>"#,
    ),
    (
        "nested_svg_dynamic_prop_boundary",
        r#"<div><svg xmlns="http://www.w3.org/2000/svg" :width="w" /></div>"#,
    ),
    (
        "svg_same_namespace_dynamic_descendants",
        r#"<div><svg><defs><pattern :x="x"><line :x1="w"/></pattern></defs></svg></div>"#,
    ),
    ("mathml_static_tree", "<math><mi>x</mi></math>"),
];

#[test]
fn s2_foreign_namespaces_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
