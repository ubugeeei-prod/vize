//! Focused Davinci parity pins for static-props/static-vnode hoist order.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom;

fn shipped(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template(&allocator, source);
    let blocking: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_compatibility_notice())
        .collect();
    assert!(blocking.is_empty(), "{source:?}: {blocking:?}");
    format!("{}\n{}", old.preamble, old.code)
}

fn emitted(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

fn assert_shipped_parity(source: &str) {
    assert_eq!(emitted(source), shipped(source), "{source}");
}

#[test]
fn nested_not_static_props_hoist_in_legacy_order() {
    assert_shipped_parity(r#"<div><section class="panel"><span>{{ msg }}</span></section></div>"#);
}

#[test]
fn dynamic_parent_hoists_static_children_instead_of_render_cache() {
    assert_shipped_parity(
        r#"<div><section class="panel" @click="ok"><span></span></section></div>"#,
    );
}

#[test]
fn cached_static_child_formats_multikey_props_like_the_shipped_snapshot() {
    assert_shipped_parity(
        r#"<div class="root">{{ msg }}<span id="cta" class="pill"></span></div>"#,
    );
}

#[test]
fn cached_static_children_array_formats_multikey_props_like_the_shipped_snapshot() {
    assert_shipped_parity(
        r#"<div class="root"><span id="hero" class="title"></span><span data-panel="intro" aria-hidden="true"></span></div>"#,
    );
}

#[test]
fn static_bind_props_hoist_with_nested_dynamic_descendants() {
    assert_shipped_parity(
        r#"<div><section class="panel" :id="'fixed'"><span>{{ msg }}</span></section></div>"#,
    );
}

#[test]
fn v_for_item_props_hoist_is_registered_but_not_used() {
    assert_shipped_parity(r#"<div v-for="item in list" class="row">{{ item }}</div>"#);
    assert_shipped_parity(
        r#"<template v-for="item in list"><section class="row"><span>{{ item }}</span></section></template>"#,
    );
}

#[test]
fn v_for_component_item_props_hoist_is_registered_but_not_used() {
    assert_shipped_parity(r#"<Foo v-for="item in list" class="row"><span>{{ item }}</span></Foo>"#);
}
