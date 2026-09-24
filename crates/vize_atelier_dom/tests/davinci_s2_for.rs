//! P2-11 destructured `v-for` aliases, compared **byte-for-byte**
//! including helper usage and fragment flags.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

#[expect(
    dead_code,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; each binary uses a subset"
)]
mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "static_key",
        r#"<div v-for="item in list" key="row">{{ item }}</div>"#,
    ),
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

#[test]
fn s2_destructured_v_for_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
