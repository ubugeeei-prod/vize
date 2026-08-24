//! P2-11 object `v-on` witness: `_toHandlers(..., true)`, mergeProps
//! with attrs / named events / object `v-bind` / v-if keys, compared
//! **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{emit_dom_source, DOM_LANE_FLAG};

const BATTERY: &[(&str, &str)] = &[
    ("object_on", r#"<div v-on="handlers"></div>"#),
    (
        "attr_then_object_on",
        r#"<div id="x" v-on="handlers"></div>"#,
    ),
    (
        "object_on_then_attr",
        r#"<div v-on="handlers" id="x"></div>"#,
    ),
    (
        "click_then_object_on",
        r#"<div @click="h" v-on="handlers"></div>"#,
    ),
    (
        "object_on_then_click",
        r#"<div v-on="handlers" @click="h"></div>"#,
    ),
    (
        "object_bind_then_object_on",
        r#"<div v-bind="obj" v-on="handlers"></div>"#,
    ),
    (
        "object_on_then_object_bind",
        r#"<div v-on="handlers" v-bind="obj"></div>"#,
    ),
    (
        "v_if_object_on",
        r#"<div v-if="ok" v-on="handlers">x</div>"#,
    ),
    ("dyn_id", r#"<div v-on="handlers" :id="x"></div>"#),
    ("class_then", r#"<div class="a" v-on="handlers"></div>"#),
    ("keyup_then", r#"<div @keyup="h" v-on="handlers"></div>"#),
    ("nested", r#"<div><span v-on="handlers"></span></div>"#),
    ("component_object_on", r#"<Foo v-on="handlers" />"#),
    (
        "component_attr_then_object_on",
        r#"<Foo id="x" v-on="handlers" />"#,
    ),
    (
        "component_click_then_object_on",
        r#"<Foo @click="h" v-on="handlers" />"#,
    ),
    ("slot_object_on", r#"<slot v-on="handlers"></slot>"#),
    (
        "fragment_object_on",
        r#"<div v-on="handlers"></div><span></span>"#,
    ),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_object_v_on_matches_the_shipped_dom_lane_byte_for_byte() {
    let mut compared = 0u64;
    let mut skipped_legacy_flag = 0u64;
    if std::env::var(DOM_LANE_FLAG).is_ok_and(|value| value == "legacy") {
        skipped_legacy_flag += 1;
    } else {
        let allocator = Allocator::new();
        for (name, src) in BATTERY {
            let old = shipped(src);
            let new = emit_dom_source(&allocator, src)
                .unwrap_or_else(|error| panic!("{name}: S2 emit refused: {error:?}"))
                .assembled();
            assert_eq!(
                old.as_str(),
                new.as_str(),
                "{name}: S2 DOM emit diverged from the shipped lane"
            );
            compared += 1;
        }
    }
    assert_eq!(
        (compared, skipped_legacy_flag),
        (BATTERY.len() as u64, 0),
        "a cfg or {DOM_LANE_FLAG}=legacy regression disarmed the dual-run"
    );
}
