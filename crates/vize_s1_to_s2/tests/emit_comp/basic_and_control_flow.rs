use super::*;

#[test]
fn a_root_component_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<Foo />"),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo))
}")
    );
}

#[test]
fn a_nested_component_uses_create_vnode() {
    assert_eq!(
        assembled("<div><Foo /></div>"),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createVNode(_component_Foo)
  ]))
}")
    );
}

#[test]
fn a_nested_keyed_component_uses_create_block() {
    assert_eq!(
        assembled(r#"<div><Foo :key="renderKey" /></div>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createElementBlock: _createElementBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createElementBlock(\"div\", null, [
    (_openBlock(), _createBlock(_component_Foo, { key: renderKey }))
  ]))
}")
    );
}

#[test]
fn a_kebab_name_underscores_the_asset_id() {
    assert_eq!(
        assembled("<foo-bar />"),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_foo_bar = _resolveComponent(\"foo-bar\")

  return (_openBlock(), _createBlock(_component_foo_bar))
}")
    );
}

#[test]
fn a_component_bind_uses_props_not_class_flag() {
    assert_eq!(
        assembled(r#"<Foo :class="cls" />"#),
        pin("\
const { resolveComponent: _resolveComponent, normalizeClass: _normalizeClass, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    class: _normalizeClass(cls)
  }, null, 8 /* PROPS */, [\"class\"]))
}")
    );
}

#[test]
fn a_component_keyup_skips_hydration() {
    assert_eq!(
        assembled(r#"<Foo @keyup="h" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, { onKeyup: h }, null, 8 /* PROPS */, [\"onKeyup\"]))
}")
    );
}

#[test]
fn a_component_v_if_uses_create_block() {
    assert_eq!(
        assembled(r#"<Foo v-if="ok" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (ok)
    ? (_openBlock(), _createBlock(_component_Foo, { key: 0 }))
    : _createCommentVNode(\"v-if\", true)
}")
    );
}

#[test]
fn a_component_v_if_branch_key_suppresses_authored_key_bind() {
    assert_eq!(
        assembled(r#"<template v-if="ok"><Foo :key="renderKey" :title="title" /></template>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (ok)
    ? (_openBlock(), _createBlock(_component_Foo, {
      key: 0,
      title: title
    }, null, 8 /* PROPS */, [\"title\"]))
    : _createCommentVNode(\"v-if\", true)
}")
    );
}

#[test]
fn duplicate_nested_components_resolve_once() {
    assert_eq!(
        assembled("<div><Foo /><Foo /></div>"),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createVNode(_component_Foo),
    _createVNode(_component_Foo)
  ]))
}")
    );
}

#[test]
fn a_v_for_item_component_omits_empty_props() {
    assert_eq!(
        assembled(r#"<Foo v-for="i in n" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
    return (_openBlock(), _createBlock(_component_Foo))
  }), 256 /* UNKEYED_FRAGMENT */))
}")
    );
}
