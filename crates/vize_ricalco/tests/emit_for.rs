//! Native `ui.for` emit pins, including `<template v-for>` fragments.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_ricalco::{EmitError, emit_dom};

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

fn refused(source: &str) -> EmitError {
    with_transformed(source, |lowered, _, facts, _| {
        emit_dom(lowered, facts).expect_err("expected Unsupported")
    })
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
fn a_static_v_for_item_hoists() {
    assert_eq!(
        assembled(r#"<div><span v-for="i in n">x</span></div>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\", null, \"x\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
      return _hoisted_1
    }), 256 /* UNKEYED_FRAGMENT */))
  ]))
}"
    );
}

#[test]
fn a_destructured_v_for_is_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<div v-for="{ id } in list" :key="id"></div>"#),
        EmitError::Unsupported
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
