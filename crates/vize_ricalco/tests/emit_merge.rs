//! Object-spread `v-bind` / `v-on` emit pins (`normalizeProps` /
//! `mergeProps` / `toHandlers`).

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
fn a_lone_object_bind_uses_normalize_props() {
    assert_eq!(
        assembled(r#"<div v-bind="obj"></div>"#),
        "\
const { normalizeProps: _normalizeProps, guardReactiveProps: _guardReactiveProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps(_guardReactiveProps(obj)), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_static_attr_before_object_bind_uses_merge_props() {
    assert_eq!(
        assembled(r#"<div id="x" v-bind="obj"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ id: \"x\" }, obj), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn an_object_bind_before_a_static_attr_keeps_author_order() {
    assert_eq!(
        assembled(r#"<div v-bind="obj" id="x"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps(obj, { id: \"x\" }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_named_bind_beside_object_bind_lists_the_dynamic_prop() {
    assert_eq!(
        assembled(r#"<div :id="foo" v-bind="obj"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ id: foo }, obj), null, 16 /* FULL_PROPS */, [\"id\"]))
}"
    );
}

#[test]
fn a_dynamic_class_beside_object_bind_skips_normalize_class() {
    assert_eq!(
        assembled(r#"<div :class="cls" v-bind="obj"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({
    class: cls
  }, obj), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn static_and_dynamic_class_before_object_bind_merge_as_an_array() {
    assert_eq!(
        assembled(r#"<div class="a" :class="cls" v-bind="obj"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({
    class: [\"a\", cls]
  }, obj), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_click_beside_object_bind_lists_on_click() {
    assert_eq!(
        assembled(r#"<div @click="h" v-bind="obj"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ onClick: h }, obj), null, 16 /* FULL_PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn a_keyup_beside_object_bind_sets_need_hydration() {
    assert_eq!(
        assembled(r#"<div @keyup="h" v-bind="obj"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ onKeyup: h }, obj), null, 48 /* FULL_PROPS, NEED_HYDRATION */, [\"onKeyup\"]))
}"
    );
}

#[test]
fn a_v_if_with_object_bind_merges_the_branch_key() {
    assert_eq!(
        assembled(r#"<div v-if="ok" v-bind="obj">x</div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", _mergeProps({ key: 0 }, obj), \"x\", 16 /* FULL_PROPS */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn two_object_binds_alone_keep_only_the_first() {
    assert_eq!(
        assembled(r#"<div v-bind="a" v-bind="b"></div>"#),
        "\
const { normalizeProps: _normalizeProps, guardReactiveProps: _guardReactiveProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps(_guardReactiveProps(a)), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn two_object_binds_with_an_attr_merge_both_spreads() {
    assert_eq!(
        assembled(r#"<div id="x" v-bind="a" v-bind="b"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ id: \"x\" }, a, b), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_static_attr_before_object_on_uses_merge_props() {
    assert_eq!(
        assembled(r#"<div id="x" v-on="handlers"></div>"#),
        "\
const { mergeProps: _mergeProps, toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ id: \"x\" }, _toHandlers(handlers, true)), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn an_object_on_before_a_static_attr_keeps_author_order() {
    assert_eq!(
        assembled(r#"<div v-on="handlers" id="x"></div>"#),
        "\
const { mergeProps: _mergeProps, toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps(_toHandlers(handlers, true), { id: \"x\" }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_click_beside_object_on_lists_on_click() {
    assert_eq!(
        assembled(r#"<div @click="h" v-on="handlers"></div>"#),
        "\
const { mergeProps: _mergeProps, toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps({ onClick: h }, _toHandlers(handlers, true)), null, 16 /* FULL_PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn an_object_bind_beside_object_on_uses_merge_props() {
    assert_eq!(
        assembled(r#"<div v-bind="obj" v-on="handlers"></div>"#),
        "\
const { mergeProps: _mergeProps, toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps(obj, _toHandlers(handlers, true)), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_v_if_with_object_on_merges_the_branch_key() {
    assert_eq!(
        assembled(r#"<div v-if="ok" v-on="handlers">x</div>"#),
        "\
const { mergeProps: _mergeProps, toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", _mergeProps({ key: 0 }, _toHandlers(handlers, true)), \"x\", 16 /* FULL_PROPS */))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn an_object_bind_with_modifiers_is_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<div v-bind.prop="obj"></div>"#),
        EmitError::Unsupported
    );
}

#[test]
fn a_v_for_spread_key_uses_a_multiline_key_object() {
    assert_eq!(
        assembled(r#"<div v-for="n in list" :key="i" v-bind="obj"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (n) => {
    return (_openBlock(), _createElementBlock(\"div\", _mergeProps({
      key: i
    }, obj), null, 16 /* FULL_PROPS */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_v_for_spread_merges_duplicate_handlers_after_the_spread() {
    assert_eq!(
        assembled(r#"<li v-for="item in items" :key="item.id" v-bind="item.props" @keydown="a" @keydown.enter.prevent="b"></li>"#),
        "\
const { withKeys: _withKeys, withModifiers: _withModifiers, mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
    return (_openBlock(), _createElementBlock(\"li\", _mergeProps({
      key: item.id
    }, item.props, {
      onKeydown: [a, _withKeys(_withModifiers(b, [\"prevent\"]), [\"enter\"])]
    }), null, 48 /* FULL_PROPS, NEED_HYDRATION */, [\"onKeydown\"]))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}
