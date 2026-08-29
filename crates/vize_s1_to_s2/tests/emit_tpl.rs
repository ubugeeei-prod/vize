//! `<template v-if>` / `<template v-for>` fragment pins.

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

#[test]
fn a_single_static_template_v_if_stays_a_fragment() {
    assert_eq!(
        assembled(r#"<template v-if="ok"><span></span></template>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
      _hoisted_1
    ], 64 /* STABLE_FRAGMENT */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_template_v_if_interpolation_uses_create_text() {
    assert_eq!(
        assembled(r#"<template v-if="ok">{{ msg }}</template>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
      _createTextVNode(_toDisplayString(msg), 1 /* TEXT */)
    ], 64 /* STABLE_FRAGMENT */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn an_empty_template_v_if_emits_null_children() {
    assert_eq!(
        assembled(r#"<template v-if="ok"></template>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, null, 64 /* STABLE_FRAGMENT */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_dynamic_template_v_if_child_unwraps() {
    assert_eq!(
        assembled(r#"<template v-if="ok"><span>{{ msg }}</span></template>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"span\", { key: 0 }, _toDisplayString(msg), 1 /* TEXT */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_keyed_template_v_for_puts_the_key_on_the_inner_fragment() {
    assert_eq!(
        assembled(r#"<template v-for="item in list" :key="item"><span></span><span></span></template>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (item) => {
    return (_openBlock(), _createElementBlock(_Fragment, { key: item }, [
      _hoisted_1,
      _hoisted_2
    ], 64 /* STABLE_FRAGMENT */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_keyed_dynamic_template_v_for_unwraps() {
    assert_eq!(
        assembled(r#"<template v-for="item in list" :key="item"><span>{{ item }}</span></template>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (item) => {
    return (_openBlock(), _createElementBlock(\"span\", { key: item }, _toDisplayString(item), 1 /* TEXT */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_template_v_if_wrapping_v_for_puts_the_key_on_the_list() {
    assert_eq!(
        assembled(r#"<template v-if="ok"><div v-for="i in n">{{ i }}</div></template>"#),
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
