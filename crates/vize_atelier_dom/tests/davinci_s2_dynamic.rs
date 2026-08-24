//! P2-11 `<component :is>` witness: `resolveDynamicComponent`, skipped
//! `is` props / patch flags, nested `createBlock`, compared
//! **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, emit_dom_source};

const BATTERY: &[(&str, &str)] = &[
    ("dynamic_bind", r#"<component :is="x" />"#),
    ("dynamic_empty", r#"<component :is="x"></component>"#),
    ("static_is", r#"<component is="Foo" />"#),
    ("pascal_bind", r#"<Component :is="x" />"#),
    ("pascal_static", r#"<Component is="Foo" />"#),
    ("bare_component", "<component />"),
    ("with_id", r#"<component :is="x" id="a" />"#),
    ("with_bind", r#"<component :is="x" :foo="bar" />"#),
    ("text_slot", r#"<component :is="x">hello</component>"#),
    (
        "span_slot",
        r#"<component :is="x"><span></span></component>"#,
    ),
    ("nested", r#"<div><component :is="x" /></div>"#),
    ("vif", r#"<component v-if="ok" :is="x" />"#),
    ("vfor", r#"<component v-for="i in n" :is="x" />"#),
    ("object_bind", r#"<component :is="x" v-bind="obj" />"#),
    ("in_slot", r#"<Foo><component :is="x" /></Foo>"#),
    (
        "static_is_slot",
        r#"<component is="Foo" id="a">hello</component>"#,
    ),
    ("quoted", r#"<component :is="'Foo'" />"#),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_dynamic_is_matches_the_shipped_dom_lane_byte_for_byte() {
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
