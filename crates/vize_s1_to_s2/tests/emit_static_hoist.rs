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
fn static_ref_child_stays_inline_when_static_cache_is_enabled() {
    assert_shipped_parity(
        r#"<aside class="seed"></aside><main><div ref="canvasContainerRef"></div></main>"#,
    );
}

#[test]
fn cached_static_children_array_formats_multikey_props_like_the_shipped_snapshot() {
    assert_shipped_parity(
        r#"<div class="root"><span id="hero" class="title"></span><span data-panel="intro" aria-hidden="true"></span></div>"#,
    );
}

#[test]
fn static_ref_child_stays_inline_under_dynamic_parent_hoist() {
    assert_shipped_parity(
        r#"<section @mousemove="track"><div ref="silhouette" class="silhouette"></div></section>"#,
    );
}

#[test]
fn static_bind_props_hoist_with_nested_dynamic_descendants() {
    assert_shipped_parity(
        r#"<div><section class="panel" :id="'fixed'"><span>{{ msg }}</span></section></div>"#,
    );
}

#[test]
fn nested_event_and_model_children_match_legacy_block_shape() {
    assert_shipped_parity(r#"<div><button @click="run">Run</button></div>"#);
    assert_shipped_parity(
        r#"<div class="password-input"><input v-model="password" :key="`password-${showPassword}`" /></div>"#,
    );
}

#[test]
fn html_parent_with_svg_child_keeps_legacy_parent_vnode_shape() {
    assert_shipped_parity(
        r#"<div><span class="menu-button" aria-label="Menu" @click="toggleMenu"><svg viewBox="0 0 24 24"><path d="M0 0h1" /></svg></span></div>"#,
    );
    assert_shipped_parity(
        r#"<label><span class="mark"><svg v-if="checked" viewBox="0 0 24 24"><path d="M0 0h1" /></svg></span></label>"#,
    );
    assert_shipped_parity(
        r#"<section><article class="chart-card"><h2>{{ title }}</h2><svg viewBox="0 0 100 40"><polyline :points="points" /></svg></article></section>"#,
    );
}

#[test]
fn cached_static_child_with_dynamic_text_uses_legacy_parent_block() {
    assert_shipped_parity(
        r#"<section><div class="cta"><svg viewBox="0 0 24 24"><path d="M0 0h1" /></svg>{{ label }}</div></section>"#,
    );
    assert_shipped_parity(
        r#"<section><a href="/docs" class="cta"><svg viewBox="0 0 24 24"><path d="M0 0h1" /></svg>{{ label }}</a></section>"#,
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
    assert_shipped_parity(
        r#"<NodeListInline v-for="document of filteredNodes" :key="document.id" :document="document" class="line-item" />"#,
    );
    assert_shipped_parity(
        r#"<NodeCard v-for="document in filteredNodes" :key="document.id" :node="document" />"#,
    );
}

#[test]
fn component_slot_dynamic_static_name_props_use_legacy_hoist() {
    assert_shipped_parity(
        r#"<NuxtLink :to="`/dashboard/docs/${document.id}`" class="name">{{ document.name }}</NuxtLink>"#,
    );
    assert_shipped_parity(
        r#"<NuxtLink :to="`/dashboard/docs/${node.id}`" class="name">{{ node.name }}</NuxtLink>"#,
    );
    assert_shipped_parity(
        r#"<footer v-if="document"><NuxtLink :to="`/dashboard/docs/edit/${document.id}`" class="edit-link">{{ document.name }}</NuxtLink></footer>"#,
    );
    assert_shipped_parity(
        r#"<TooltipRoot v-for="{ name } of contributors" :key="name"><span>{{ name }}</span></TooltipRoot>"#,
    );
    assert_shipped_parity(
        r#"<NodeResourceInline v-for="diagram in diagrams" :key="diagram.id" :node="diagram" class="line-item" />"#,
    );
}
