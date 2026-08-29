//! Named / scoped `<template>` slot emit pins (`withCtx` keys / `_`).

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
fn a_named_header_slot_uses_with_ctx() {
    assert_eq!(
        assembled("<Foo><template #header>title</template></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    header: _withCtx(() => [
      _createTextVNode(\"title\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_named_slot_concatenates_text_and_interpolation() {
    assert_eq!(
        assembled("<Foo><template #header>hello {{ msg }}</template></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, toDisplayString: _toDisplayString, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    header: _withCtx(() => [
      _createTextVNode(\"hello \" + _toDisplayString(msg), 1 /* TEXT */)
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn named_and_implicit_default_share_the_object() {
    assert_eq!(
        assembled("<Foo>hello<template #header>title</template></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    header: _withCtx(() => [
      _createTextVNode(\"title\")
    ]),
    default: _withCtx(() => [
      _createTextVNode(\"hello\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_hyphenated_slot_name_is_quoted() {
    assert_eq!(
        assembled("<Foo><template #foo-bar>x</template></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    \"foo-bar\": _withCtx(() => [
      _createTextVNode(\"x\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_dynamic_slot_name_sets_dynamic_slots() {
    assert_eq!(
        assembled(r#"<Foo><template #[name]>x</template></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    [name]: _withCtx(() => [
      _createTextVNode(\"x\")
    ]),
    _: 2 /* DYNAMIC */
  }, 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn a_scoped_slot_passes_the_param() {
    assert_eq!(
        assembled(r#"<Foo><template #header="p">x</template></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    header: _withCtx((p) => [
      _createTextVNode(\"x\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_component_root_named_v_slot_preserves_its_key() {
    assert_eq!(
        assembled("<Foo v-slot:header>title</Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    header: _withCtx(() => [
      _createTextVNode(\"title\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_component_root_bare_v_slot_keys_default() {
    assert_eq!(
        assembled("<Foo v-slot>title</Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _createTextVNode(\"title\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}
