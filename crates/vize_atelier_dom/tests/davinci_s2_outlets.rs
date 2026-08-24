//! P2-11 slot-outlet witness: `_renderSlot`, fallback, camelized props,
//! `v-if` / `v-for` outlets, and `_: 3 /* FORWARDED */`, compared
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
    ("bare", "<slot></slot>"),
    ("self_close", "<slot/>"),
    ("named", r#"<slot name="header"></slot>"#),
    ("dynamic_name", r#"<slot :name="n"></slot>"#),
    ("fallback_text", "<slot>fallback</slot>"),
    ("fallback_interp", "<slot>hello {{ msg }}</slot>"),
    ("fallback_span", "<slot><span></span></slot>"),
    ("static_prop", r#"<slot foo="bar"></slot>"#),
    ("hyphen_prop", r#"<slot foo-bar="x"></slot>"#),
    ("bind_prop", r#"<slot :foo="bar"></slot>"#),
    ("bind_hyphen", r#"<slot :foo-bar="x"></slot>"#),
    ("prop_and_fallback", r#"<slot foo="bar">fb</slot>"#),
    ("object_bind", r#"<slot v-bind="obj"></slot>"#),
    ("in_div", "<div><slot></slot></div>"),
    ("forwarded", "<Foo><slot></slot></Foo>"),
    ("forwarded_nested", "<Foo><div><slot></slot></div></Foo>"),
    ("vif", r#"<slot v-if="ok"></slot>"#),
    ("vif_fallback", r#"<slot v-if="ok">x</slot>"#),
    ("vif_else", r#"<slot v-if="a"></slot><slot v-else></slot>"#),
    ("vfor", r#"<slot v-for="i in n"></slot>"#),
    ("vfor_fallback", r#"<slot v-for="i in n">x</slot>"#),
    (
        "scoped_forwarded",
        r#"<Bar v-slot="p"><Foo><slot></slot></Foo></Bar>"#,
    ),
    (
        "named_mixed_props",
        r#"<slot name="header" foo="1" :bar="b"></slot>"#,
    ),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_slot_outlets_match_the_shipped_dom_lane_byte_for_byte() {
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
