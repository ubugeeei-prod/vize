//! Custom `vue.directive` emit pins (`resolveDirective` / `withDirectives`).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_ricalco::emit_dom;

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
fn an_empty_unique_custom_dir_sets_need_patch() {
    assert_eq!(
        assembled(r#"<div v-example></div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return _withDirectives((_openBlock(), _createElementBlock(\"div\", null, null, 512 /* NEED_PATCH */)), [
    [_directive_example]
  ])
}")
    );
}

#[test]
fn unique_text_children_force_an_array() {
    assert_eq!(
        assembled(r#"<div v-example>hello</div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return _withDirectives((_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(\"hello\")
  ], 512 /* NEED_PATCH */)), [
    [_directive_example]
  ])
}")
    );
}

#[test]
fn unique_interp_combines_text_and_need_patch() {
    assert_eq!(
        assembled(r#"<div v-example>{{ msg }}</div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return _withDirectives((_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(_toDisplayString(msg), 1 /* TEXT */)
  ], 513 /* TEXT, NEED_PATCH */)), [
    [_directive_example]
  ])
}")
    );
}

#[test]
fn a_value_arg_and_modifiers_match_the_entry_shape() {
    assert_eq!(
        assembled(r#"<div v-example:arg.foo="val"></div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return _withDirectives((_openBlock(), _createElementBlock(\"div\", null, null, 512 /* NEED_PATCH */)), [
    [_directive_example, val, \"arg\", { foo: true }]
  ])
}")
    );
}

#[test]
fn a_kebab_name_becomes_an_underscore_ident() {
    assert_eq!(
        assembled(r#"<div v-my-dir></div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_my_dir = _resolveDirective(\"my-dir\")

  return _withDirectives((_openBlock(), _createElementBlock(\"div\", null, null, 512 /* NEED_PATCH */)), [
    [_directive_my_dir]
  ])
}")
    );
}

#[test]
fn a_dynamic_prop_drops_need_patch() {
    assert_eq!(
        assembled(r#"<div :id="id" v-example></div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return _withDirectives((_openBlock(), _createElementBlock(\"div\", { id: id }, null, 8 /* PROPS */, [\"id\"])), [
    [_directive_example]
  ])
}")
    );
}

#[test]
fn nested_text_stays_inlined() {
    assert_eq!(
        assembled(r#"<div><span v-example>hello</span></div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return (_openBlock(), _createElementBlock(\"div\", null, [
    _withDirectives(_createElementVNode(\"span\", null, \"hello\", 512 /* NEED_PATCH */), [
      [_directive_example]
    ])
  ]))
}")
    );
}

#[test]
fn a_v_for_item_emits_empty_props_without_need_patch() {
    assert_eq!(
        assembled(r#"<div v-for="i in n" v-example></div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
    return _withDirectives((_openBlock(), _createElementBlock(\"div\", { })), [
      [_directive_example]
    ])
  }), 256 /* UNKEYED_FRAGMENT */))
}")
    );
}

#[test]
fn a_keyed_v_for_item_uses_a_multiline_key_object() {
    assert_eq!(
        assembled(r#"<div v-for="i in n" :key="i" v-example></div>"#),
        pin("\
const { resolveDirective: _resolveDirective, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
    return _withDirectives((_openBlock(), _createElementBlock(\"div\", {
      key: i
    })), [
      [_directive_example]
    ])
  }), 128 /* KEYED_FRAGMENT */))
}")
    );
}

#[test]
fn a_v_for_component_uses_empty_props_object() {
    assert_eq!(
        assembled(r#"<Foo v-for="i in n" v-example />"#),
        pin("\
const { resolveDirective: _resolveDirective, resolveComponent: _resolveComponent, withDirectives: _withDirectives, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")
  const _directive_example = _resolveDirective(\"example\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
    return _withDirectives((_openBlock(), _createBlock(_component_Foo, { })), [
      [_directive_example]
    ])
  }), 256 /* UNKEYED_FRAGMENT */))
}")
    );
}

#[test]
fn a_component_resolves_the_directive_before_the_tag() {
    assert_eq!(
        assembled(r#"<Foo v-example />"#),
        pin("\
const { resolveDirective: _resolveDirective, resolveComponent: _resolveComponent, withDirectives: _withDirectives, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")
  const _directive_example = _resolveDirective(\"example\")

  return _withDirectives((_openBlock(), _createBlock(_component_Foo, null, null, 512 /* NEED_PATCH */)), [
    [_directive_example]
  ])
}")
    );
}

#[test]
fn native_v_model_keeps_the_model_entry_first() {
    assert_eq!(
        assembled(r#"<input v-example v-model="x">"#),
        pin("\
const { resolveDirective: _resolveDirective, vModelText: _vModelText, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return _withDirectives((_openBlock(), _createElementBlock(\"input\", {
    \"onUpdate:modelValue\": $event => ((x) = $event)
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"])), [
    [_vModelText, x],
    [_directive_example]
  ])
}")
    );
}
