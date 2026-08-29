//! P2-11 template fragments: empty roots, unique text, multi-root
//! `_Fragment` + `STABLE_FRAGMENT`, and compound interpolations
//! expanded as generate_node children.

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

fn pin(visual: &str) -> String {
    visual.replace(")\n\n  return", ")\n  \n  return")
}

#[test]
fn empty_root_returns_null() {
    assert_eq!(
        assembled(""),
        "\
\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {
  return null
}"
    );
}

#[test]
fn whitespace_only_root_returns_null() {
    assert_eq!(assembled("   \n"), assembled(""));
}

#[test]
fn unique_root_text_is_a_text_vnode() {
    assert_eq!(
        assembled("hello"),
        "\
const { createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _createTextVNode(\"hello\")
}"
    );
}

#[test]
fn two_native_roots_wrap_in_a_stable_fragment() {
    assert_eq!(
        assembled("<div></div><span></span>"),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createElementVNode(\"div\"),
    _createElementVNode(\"span\")
  ], 64 /* STABLE_FRAGMENT */))
}"
    );
}

#[test]
fn root_static_props_hoist_per_fragment_child() {
    assert_eq!(
        assembled(r#"<div class="x"></div><span id="y"></span>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment } = Vue

const _hoisted_1 = { class: \"x\" }
const _hoisted_2 = { id: \"y\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createElementVNode(\"div\", _hoisted_1),
    _createElementVNode(\"span\", _hoisted_2)
  ], 64 /* STABLE_FRAGMENT */))
}"
    );
}

#[test]
fn compound_root_text_expands_into_a_fragment() {
    assert_eq!(
        assembled("hello {{ msg }}"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createTextVNode(\"hello \"),
    _toDisplayString(msg)
  ], 64 /* STABLE_FRAGMENT */))
}"
    );
}

#[test]
fn two_components_are_nested_vnodes_inside_the_fragment() {
    assert_eq!(
        assembled("<Foo /><Bar />"),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")
  const _component_Bar = _resolveComponent(\"Bar\")

  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createVNode(_component_Foo),
    _createVNode(_component_Bar)
  ], 64 /* STABLE_FRAGMENT */))
}")
    );
}

#[test]
fn sibling_v_if_roots_share_one_fragment() {
    assert_eq!(
        assembled(r#"<p v-if="a">1</p><span v-if="b">2</span>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(_Fragment, null, [
    (a)
      ? (_openBlock(), _createElementBlock(\"p\", { key: 0 }, \"1\"))
      : _createCommentVNode(\"v-if\", true),
    (b)
      ? (_openBlock(), _createElementBlock(\"span\", { key: 1 }, \"2\"))
      : _createCommentVNode(\"v-if\", true)
  ], 64 /* STABLE_FRAGMENT */))
}"
    );
}
