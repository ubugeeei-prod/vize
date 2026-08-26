//! P2-11 installment 5: static native HTML, interpolations, mixed
//! text siblings, and static-name binds emit the same render function
//! the shipped DOM lane does.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_ricalco::{EmitError, UnsupportedReason as Reason, emit_dom, emit_dom_source};
use vize_s0::Allocator;

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

#[test]
fn empty_div_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div></div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\"))
}"
    );
}

#[test]
fn div_with_text_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div>hello</div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, \"hello\"))
}"
    );
}

#[test]
fn nested_elements_match_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div><span>hello</span></div>"),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", null, \"hello\")
  ]))
}"
    );
}

#[test]
fn emit_dom_source_agrees_with_emit_dom() {
    let allocator = Allocator::new();
    let via_source = emit_dom_source(&allocator, "<p>hi</p>").expect("emit");
    assert_eq!(assembled("<p>hi</p>"), via_source.assembled().as_str());
}

#[test]
fn empty_div_with_class_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div class="x"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { class: \"x\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn multiple_static_attrs_hoist_as_one_object() {
    assert_eq!(
        assembled(r#"<div id="app" class="container">static</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { id: \"app\", class: \"container\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1, \"static\"))
}"
    );
}

#[test]
fn hyphenated_attr_names_are_quoted() {
    assert_eq!(
        assembled(r#"<div data-id="1"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { \"data-id\": \"1\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn boolean_attr_emits_an_empty_string_value() {
    assert_eq!(
        assembled("<div disabled></div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { disabled: \"\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn nested_static_attrs_match_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div><span class="x">hello</span></div>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", { class: \"x\" }, \"hello\")
  ]))
}"
    );
}

#[test]
fn a_bound_class_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div :class="cls"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(cls)
  }, null, 2 /* CLASS */))
}"
    );
}

#[test]
fn a_bound_id_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div :id="foo"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { id: foo }, null, 8 /* PROPS */, [\"id\"]))
}"
    );
}

#[test]
fn simple_interpolation_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("{{ msg }}"),
        "\
const { toDisplayString: _toDisplayString } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _toDisplayString(msg)
}"
    );
}

#[test]
fn interpolation_in_element_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div>{{ msg }}</div>"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(msg), 1 /* TEXT */))
}"
    );
}

#[test]
fn mixed_text_and_interpolation_compiles_from_text_facts() {
    assert_eq!(
        assembled("<div>hello {{ msg }}</div>"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, \"hello \" + _toDisplayString(msg), 1 /* TEXT */))
}"
    );
}

#[test]
fn hoisted_static_props_omit_the_text_patch_flag() {
    assert_eq!(
        assembled(r#"<div class="x">{{ msg }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { class: \"x\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1, _toDisplayString(msg)))
}"
    );
}

#[test]
fn nested_interpolation_keeps_the_text_patch_flag() {
    assert_eq!(
        assembled("<div><span>{{ msg }}</span></div>"),
        "\
const { toDisplayString: _toDisplayString, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", null, _toDisplayString(msg), 1 /* TEXT */)
  ]))
}"
    );
}

#[test]
fn a_trailing_root_newline_does_not_steal_compound_child_ids() {
    assert_eq!(
        assembled("<div>Hi {{ name }}</div>\n"),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, \"Hi \" + _toDisplayString(name), 1 /* TEXT */))
}"
    );
}

#[test]
fn mixed_element_and_interpolation_siblings_use_create_text_vnode() {
    assert_eq!(
        assembled("<div>{{ msg }}<span></span></div>"),
        "\
const { toDisplayString: _toDisplayString, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(_toDisplayString(msg), 1 /* TEXT */),
    _createElementVNode(\"span\")
  ]))
}"
    );
}

#[test]
fn mixed_static_text_and_element_siblings_use_create_text_vnode() {
    assert_eq!(
        assembled("<div>hello<span></span></div>"),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(\"hello\"),
    _createElementVNode(\"span\")
  ]))
}"
    );
}

#[test]
fn a_single_space_between_elements_is_create_text_vnode_with_no_args() {
    assert_eq!(
        assembled("<div><span></span> <span></span></div>"),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\"),
    _createTextVNode(),
    _createElementVNode(\"span\")
  ]))
}"
    );
}

fn refused(source: &str) -> EmitError {
    with_transformed(source, |lowered, _, facts, _| {
        emit_dom(lowered, facts).expect_err("expected Unsupported")
    })
}

#[test]
fn v_once_wraps_the_native_vnode_in_the_render_cache() {
    assert_eq!(
        assembled("<div v-once>x</div>"),
        "\
const { createElementVNode: _createElementVNode, createTextVNode: _createTextVNode, setBlockTracking: _setBlockTracking } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _cache[0] || (
    _setBlockTracking(-1, true),
    (_cache[0] = _createElementVNode(\"div\", null, [
      _createTextVNode(\"x\")
    ])).cacheIndex = 0,
    _setBlockTracking(1),
    _cache[0]
  )
}"
    );
}

#[test]
fn v_memo_is_unsupported_until_realization() {
    assert_eq!(
        refused(r#"<div v-memo="[id]">x</div>"#).reason(),
        Some(Reason::UnsupportedBindingKind)
    );
}
