//! Native `ui.for` emit pins, including `<template v-for>` fragments.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom;

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

/// Vue's extra `newline()` after `genAssets` leaves indent on the blank line.
fn pin(visual: &str) -> String {
    visual.replace(")\n\n  return", ")\n  \n  return")
}

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

fn assert_shipped_parity(source: &str) {
    assert_eq!(assembled(source), shipped(source), "{source}");
}

#[test]
fn a_keyed_v_for_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div v-for="item in list" :key="item">{{ item }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (item) => {
    return (_openBlock(), _createElementBlock(\"div\", { key: item }, _toDisplayString(item), 1 /* TEXT */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn an_unkeyed_v_for_sets_unkeyed_fragment() {
    assert_eq!(
        assembled(r#"<div v-for="item in list">{{ item }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (item) => {
    return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(item), 1 /* TEXT */))
  }), 256 /* UNKEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_numeric_v_for_is_a_stable_fragment() {
    assert_eq!(
        assembled(r#"<div v-for="n in 3">{{ n }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(_Fragment, null, _renderList(3, (n) => {
    return _createElementVNode(\"div\", null, _toDisplayString(n), 1 /* TEXT */)
  }), 64 /* STABLE_FRAGMENT */))
}"
    );
}

#[test]
fn a_static_v_for_item_stays_a_block() {
    assert_eq!(
        assembled(r#"<div><span v-for="i in n">x</span></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
      return (_openBlock(), _createElementBlock(\"span\", null, \"x\"))
    }), 256 /* UNKEYED_FRAGMENT */))
  ]))
}"
    );
}

#[test]
fn an_object_destructured_v_for_emits_the_pattern() {
    assert_eq!(
        assembled(r#"<div v-for="{ id } in list" :key="id">{{ id }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, ({ id }) => {
    return (_openBlock(), _createElementBlock(\"div\", { key: id }, _toDisplayString(id), 1 /* TEXT */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn an_array_destructured_v_for_emits_the_pattern() {
    assert_eq!(
        assembled(r#"<div v-for="[a, b] in list">{{ a }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, ([a, b]) => {
    return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(a), 1 /* TEXT */))
  }), 256 /* UNKEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_defaulted_destructure_emits_the_authored_pattern() {
    assert_eq!(
        assembled(r#"<div v-for="{ id = 1 } in list" :key="id">{{ id }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, ({ id = 1 }) => {
    return (_openBlock(), _createElementBlock(\"div\", { key: id }, _toDisplayString(id), 1 /* TEXT */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_middle_hole_drops_the_empty_key_alias() {
    assert_eq!(
        assembled(r#"<div v-for="(item, , i) in list" :key="i">{{ item }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (item, i) => {
    return (_openBlock(), _createElementBlock(\"div\", { key: i }, _toDisplayString(item), 1 /* TEXT */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_destructured_v_for_component_uses_the_pattern() {
    assert_eq!(
        assembled(r#"<Foo v-for="{ id } in list" :key="id" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, ({ id }) => {
    return (_openBlock(), _createBlock(_component_Foo, { key: id }))
  }), 128 /* KEYED_FRAGMENT */))
}")
    );
}

#[test]
fn a_template_v_for_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<template v-for="item in list"><span></span><span></span></template>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (item) => {
    return (_openBlock(), _createElementBlock(_Fragment, null, [
      _hoisted_1,
      _hoisted_2
    ], 64 /* STABLE_FRAGMENT */))
  }), 256 /* UNKEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_template_v_for_child_key_does_not_emit_on_the_unwrapped_root() {
    assert_shipped_parity(
        r#"<template v-for="(point, i) in points"><rect :key="i" :fill="color"><animate v-if="ok" /></rect></template>"#,
    );
}

#[test]
fn template_v_for_unwrapped_root_drops_runtime_directive_wrappers() {
    assert_shipped_parity(
        r#"<template v-for="item in blocks"><div :key="item.id" v-masonry-tile :class="item.class" class="grid"><span></span></div></template>"#,
    );
    assert_shipped_parity(
        r#"<template v-for="[fullPath, Comp] in compList" :key="fullPath"><div v-show="fullPath === currRoute.fullPath" class="size-full"><slot :fullPath="fullPath" :Comp="Comp" /></div></template>"#,
    );
    assert_shipped_parity(
        r#"<template v-for="item in items"><input v-model="item.name" :key="item.id" /></template>"#,
    );
}

#[test]
fn template_v_for_nested_if_root_keeps_runtime_directive_wrappers() {
    assert_shipped_parity(
        r#"<template v-for="item in items"><div v-if="item.ok" v-show="item.ok" :key="item.id"></div></template>"#,
    );
    assert_shipped_parity(
        r#"<template v-for="item in items"><div><span v-show="item.ok"></span><span v-ripple></span></div></template>"#,
    );
}
