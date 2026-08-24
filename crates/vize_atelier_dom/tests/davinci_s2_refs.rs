//! P2-11 template-ref witness for the S2 DOM lane: static refs,
//! dynamic `:ref`, `ref_for` inside `v-for`, and component refs are
//! compared **byte-for-byte** against the shipped DOM lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("static_ref", r#"<div ref="el"></div>"#),
    ("dynamic_ref", r#"<div :ref="el"></div>"#),
    ("static_ref_interp", r#"<div ref="el">{{ msg }}</div>"#),
    ("dynamic_ref_interp", r#"<div :ref="el">{{ msg }}</div>"#),
    ("static_ref_dynamic_id", r#"<div ref="el" :id="id"></div>"#),
    (
        "dynamic_ref_dynamic_id",
        r#"<div :ref="el" :id="id"></div>"#,
    ),
    ("static_ref_click", r#"<div ref="el" @click="h"></div>"#),
    ("static_ref_for", r#"<div v-for="i in n" ref="el"></div>"#),
    ("dynamic_ref_for", r#"<div v-for="i in n" :ref="el"></div>"#),
    (
        "dynamic_ref_for_with_static_prop",
        r#"<div v-for="i in n" id="x" :ref="el"></div>"#,
    ),
    (
        "dynamic_ref_for_keyed",
        r#"<div v-for="i in n" :key="i" :ref="el"></div>"#,
    ),
    ("component_static_ref", r#"<Foo ref="el" />"#),
    ("component_dynamic_ref", r#"<Foo :ref="el" />"#),
    (
        "component_static_ref_dynamic_id",
        r#"<Foo ref="el" :id="id" />"#,
    ),
    (
        "static_ref_object_bind",
        r#"<div ref="el" v-bind="obj"></div>"#,
    ),
    (
        "dynamic_ref_object_bind",
        r#"<div :ref="el" v-bind="obj"></div>"#,
    ),
    (
        "component_dynamic_ref_object_bind",
        r#"<Foo :ref="el" v-bind="obj" />"#,
    ),
    (
        "component_dynamic_ref_for",
        r#"<Foo v-for="i in n" :ref="el" />"#,
    ),
];

#[test]
fn s2_template_refs_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
