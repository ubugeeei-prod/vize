//! P2-11 destructured `v-for` aliases, compared **byte-for-byte**
//! including helper usage and fragment flags.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, emit_dom_source};

const BATTERY: &[(&str, &str)] = &[
    (
        "obj",
        r#"<div v-for="{ id } in list" :key="id">{{ id }}</div>"#,
    ),
    (
        "obj_idx",
        r#"<div v-for="({ id, name }, i) in list" :key="id">{{ name }}</div>"#,
    ),
    ("arr", r#"<div v-for="[a, b] in list">{{ a }}</div>"#),
    (
        "nested",
        r#"<div v-for="{ user: { name } } in list">{{ name }}</div>"#,
    ),
    (
        "rest",
        r#"<div v-for="{ id, ...rest } in list" :key="id">{{ id }}</div>"#,
    ),
    (
        "default",
        r#"<div v-for="{ id = 1 } in list" :key="id">{{ id }}</div>"#,
    ),
    (
        "rename",
        r#"<div v-for="{ id: rowId } in list" :key="rowId">{{ rowId }}</div>"#,
    ),
    (
        "hole",
        r#"<div v-for="(item, , i) in list" :key="i">{{ item }}</div>"#,
    ),
    (
        "tpl",
        r#"<template v-for="{ id } in list" :key="id"><span>{{ id }}</span></template>"#,
    ),
    ("comp", r#"<Foo v-for="{ id } in list" :key="id" />"#),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_destructured_v_for_matches_the_shipped_dom_lane_byte_for_byte() {
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
