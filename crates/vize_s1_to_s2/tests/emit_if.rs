//! Native `ui.if` emit pins, including `<template v-if>` fragments.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
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

#[test]
fn a_root_v_if_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div v-if="ok">hello</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: 0 }, \"hello\"))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_root_v_if_else_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div v-if="ok">yes</div><div v-else>no</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: 0 }, \"yes\"))
    : (_openBlock(), _createElementBlock(\"div\", { key: 1 }, \"no\"))
}"
    );
}

#[test]
fn a_static_branch_key_is_quoted() {
    assert_eq!(
        assembled(r#"<div v-if="ok" key="k"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: \"k\" }))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn mixed_text_in_a_v_if_lists_comment_before_text() {
    assert_eq!(
        assembled(r#"<div v-if="ok"><span></span> x</div>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode, createTextVNode: _createTextVNode } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: 0 }, [
      _hoisted_1,
      _createTextVNode(\" x\")
    ]))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_dynamic_branch_key_emits_the_expression() {
    assert_eq!(
        assembled(r#"<div v-if="ok" :key="k"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: k }))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_valueless_dynamic_key_expands_to_the_name() {
    assert_eq!(
        assembled(r#"<div v-if="ok" :key></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: key }))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_dynamic_key_on_a_component_stays_on_the_props() {
    assert_eq!(
        assembled(r#"<Foo v-if="ok" :key="k" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (ok)
    ? (_openBlock(), _createBlock(_component_Foo, { key: k }))
    : _createCommentVNode(\"v-if\", true)
}")
    );
}

#[test]
fn a_v_if_v_for_branch_keys_the_list_fragment() {
    assert_eq!(
        assembled(r#"<div v-if="ok" v-for="i in n">{{ i }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(true), _createElementBlock(_Fragment, { key: 0 }, _renderList(n, (i) => {
      return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(i), 1 /* TEXT */))
    }), 256 /* UNKEYED_FRAGMENT */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_keyed_v_if_v_for_keeps_the_item_key() {
    assert_eq!(
        assembled(r#"<div v-if="ok" v-for="i in n" :key="i">{{ i }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(true), _createElementBlock(_Fragment, { key: 0 }, _renderList(n, (i) => {
      return (_openBlock(), _createElementBlock(\"div\", { key: i }, _toDisplayString(i), 1 /* TEXT */))
    }), 128 /* KEYED_FRAGMENT */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_template_fragment_v_if_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<template v-if="ok"><span></span><span></span></template>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
      _hoisted_1,
      _hoisted_2
    ], 64 /* STABLE_FRAGMENT */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}
