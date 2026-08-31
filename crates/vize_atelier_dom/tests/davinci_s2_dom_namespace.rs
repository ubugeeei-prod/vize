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
    (
        "svg_sparkline_pulse_trail_condition",
        r#"<svg><g v-if="canRender"><template v-for="(_, i) in TRAIL_LENGTH"><circle v-if="i % 3 === 0 /* perf optimization */" :key="`sparkline_dot_${i}_${pulsePathId}`" :r="getRadius(i)"></circle></template></g></svg>"#,
    ),
    (
        "v_if_static_svg_child",
        r#"<div v-if="ok"><svg><path d="M0 0h1v1z"/></svg></div>"#,
    ),
    (
        "v_if_static_svg_foreign_object_child",
        r#"<div v-if="ok"><svg><foreignObject><div>label</div></foreignObject><path d="M0 0h1v1z"/></svg></div>"#,
    ),
    (
        "svg_foreign_object_dynamic_html_before_static_svg_sibling",
        r#"<svg><foreignObject><div v-if="ok"><span></span></div></foreignObject><rect></rect></svg>"#,
    ),
    (
        "v_show_static_svg_child",
        r#"<div v-show="ok"><svg><path d="M0 0h1v1z"/></svg></div>"#,
    ),
    (
        "component_slot_static_svg_child",
        r#"<Foo><svg><path d="M0 0h1v1z"/></svg></Foo>"#,
    ),
    ("mathml_static_tree", "<math><mi>x</mi></math>"),
    (
        "v_if_static_mathml_child",
        r#"<div v-if="ok"><math><mi>x</mi></math></div>"#,
    ),
    (
        "mathml_same_namespace_dynamic_descendants",
        r#"<div><math><mrow><msub :data-depth="depth"><mi>x</mi></msub></mrow></math></div>"#,
    ),
    (
        "mathml_annotation_xml_html_boundary",
        r#"<math><annotation-xml><div v-if="ok"><span>label</span></div></annotation-xml><mn>1</mn></math>"#,
    ),
    (
        "component_slot_static_mathml_child",
        r#"<Foo><math><mrow><mi>x</mi><mo>+</mo><mi>y</mi></mrow></math></Foo>"#,
    ),
];

#[test]
fn s2_foreign_namespaces_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
