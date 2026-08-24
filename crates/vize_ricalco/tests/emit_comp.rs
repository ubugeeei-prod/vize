//! Static-name component emit pins (`resolveComponent` / `createBlock` /
//! `createVNode`).

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
fn a_v_for_item_component_keeps_null_props() {
    assert_eq!(
        assembled(r#"<Foo v-for="i in n" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
    return (_openBlock(), _createBlock(_component_Foo, null))
  }), 256 /* UNKEYED_FRAGMENT */))
}")
    );
}

#[test]
fn a_root_transition_uses_the_builtin_helper() {
    assert_eq!(
        assembled("<Transition />"),
        pin("\
const { openBlock: _openBlock, createBlock: _createBlock, Transition: _Transition } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock(_Transition))
}")
    );
}

#[test]
fn a_dynamic_is_uses_resolve_dynamic_component() {
    assert_eq!(
        assembled(r#"<component :is="x" />"#),
        pin("\
const { resolveDynamicComponent: _resolveDynamicComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(x)))
}")
    );
}

#[test]
fn a_component_object_bind_uses_normalize_props() {
    assert_eq!(
        assembled(r#"<Foo v-bind="obj" />"#),
        pin("\
const { resolveComponent: _resolveComponent, normalizeProps: _normalizeProps, guardReactiveProps: _guardReactiveProps, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, _normalizeProps(_guardReactiveProps(obj)), null, 16 /* FULL_PROPS */))
}")
    );
}
