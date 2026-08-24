//! P2-11 colon / vnode-hook events and merged duplicate handlers,
//! compared **byte-for-byte** including helper usage and patch flags.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, emit_dom_source};

const BATTERY: &[(&str, &str)] = &[
    ("upd_comp", r#"<Foo @update:modelValue="h" />"#),
    ("upd_div", r#"<div @update:modelValue="h"></div>"#),
    ("upd_foo", r#"<div @update:foo="h"></div>"#),
    ("upd_Foo", r#"<div @update:Foo="h"></div>"#),
    ("custom_el", r#"<div @customEvent="h"></div>"#),
    ("custom_comp", r#"<Foo @customEvent="h" />"#),
    ("vue_mounted", r#"<div @vue:mounted="h"></div>"#),
    ("vue_comp", r#"<Foo @vue:mounted="h" />"#),
    ("vnode_hook", r#"<div @vnode-before-mount="h"></div>"#),
    (
        "model_then",
        r#"<Foo v-model="x" @update:modelValue="h" />"#,
    ),
    (
        "then_model",
        r#"<Foo @update:modelValue="h" v-model="x" />"#,
    ),
    ("dup_click", r#"<div @click="a" @click="b"></div>"#),
    ("click_ctrl", r#"<div @click="a" @click.ctrl="b"></div>"#),
    ("input_upd", r#"<input v-model="x" @update:modelValue="h">"#),
    (
        "named_model",
        r#"<Foo v-model:title="t" @update:title="h" />"#,
    ),
    (
        "vif_dup",
        r#"<button v-if="ok" @click="a" @click.ctrl="b"></button>"#,
    ),
    (
        "vfor_spread_dup",
        r#"<li v-for="item in items" :key="item.id" v-bind="item.props" @keydown="a" @keydown.enter.prevent="b"></li>"#,
    ),
    ("click_once", r#"<div @click.once="h"></div>"#),
    (
        "click_and_once",
        r#"<div @click="a" @click.once="b"></div>"#,
    ),
    ("nested_upd", r#"<div><Foo @update:modelValue="h" /></div>"#),
    (
        "simple_spread_key",
        r#"<div v-for="n in list" :key="i" v-bind="obj"></div>"#,
    ),
    ("kebab_el", r#"<div @foo-bar="h"></div>"#),
    ("kebab_comp", r#"<Foo @foo-bar="h" />"#),
    ("dup_comp", r#"<Foo @click="a" @click="b" />"#),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_colon_events_match_the_shipped_dom_lane_byte_for_byte() {
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
