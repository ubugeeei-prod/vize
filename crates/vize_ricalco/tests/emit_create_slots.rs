//! `createSlots` emit pins (`v-if` / `v-for` slot templates).

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
fn a_v_if_named_template_uses_create_slots() {
    assert_eq!(
        assembled(r#"<Foo><template #header v-if="ok">x</template></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, createSlots: _createSlots, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, _createSlots({ _: 2 /* DYNAMIC */ }, [
    (ok)
      ? {
        name: \"header\",
        fn: _withCtx(() => [
          _createTextVNode(\"x\")
        ]),
        key: \"0\"
      }
    : undefined
  ]), 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn a_v_for_named_template_uses_render_list() {
    assert_eq!(
        assembled(r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, renderList: _renderList, createSlots: _createSlots, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, _createSlots({ _: 2 /* DYNAMIC */ }, [
    _renderList(n, (i) => {
      return {
        name: \"header\",
        fn: _withCtx(() => [
          _createTextVNode(\"x\")
        ])
      }
    })
  ]), 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn static_named_siblings_join_the_create_slots_array() {
    assert_eq!(
        assembled(
            r#"<Foo><template #header v-if="ok">x</template><template #footer>end</template></Foo>"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, createSlots: _createSlots, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, _createSlots({ _: 2 /* DYNAMIC */ }, [
    (ok)
      ? {
        name: \"header\",
        fn: _withCtx(() => [
          _createTextVNode(\"x\")
        ]),
        key: \"0\"
      }
    : undefined,
    {
      name: \"footer\",
      fn: _withCtx(() => [
        _createTextVNode(\"end\")
      ])
    }
  ]), 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn implicit_default_text_stays_on_the_create_slots_base() {
    assert_eq!(
        assembled(r#"<Foo>hello<template #header v-if="ok">x</template></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, createSlots: _createSlots, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, _createSlots({
    default: _withCtx(() => [
      _createTextVNode(\"hello\")
    ]),
    _: 2 /* DYNAMIC */
  }, [
    (ok)
      ? {
        name: \"header\",
        fn: _withCtx(() => [
          _createTextVNode(\"x\")
        ]),
        key: \"0\"
      }
    : undefined
  ]), 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn a_v_slots_spread_lands_on_the_create_slots_base() {
    assert_eq!(
        assembled(r#"<Foo v-slots="slots"><template #header v-if="ok">x</template></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, createSlots: _createSlots, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, _createSlots({
    ...slots,
    _: 2 /* DYNAMIC */
  }, [
    (ok)
      ? {
        name: \"header\",
        fn: _withCtx(() => [
          _createTextVNode(\"x\")
        ]),
        key: \"0\"
      }
    : undefined
  ]), 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn a_v_slots_spread_follows_the_implicit_default() {
    assert_eq!(
        assembled(r#"<Foo v-slots="slots">hello<template #header v-if="ok">x</template></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, createSlots: _createSlots, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, _createSlots({
    default: _withCtx(() => [
      _createTextVNode(\"hello\")
    ]),
    ...slots,
    _: 2 /* DYNAMIC */
  }, [
    (ok)
      ? {
        name: \"header\",
        fn: _withCtx(() => [
          _createTextVNode(\"x\")
        ]),
        key: \"0\"
      }
    : undefined
  ]), 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn a_v_else_branch_omits_the_trailing_undefined() {
    assert_eq!(
        assembled(
            r#"<Foo><template #header v-if="a">x</template><template #header v-else>y</template></Foo>"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, createSlots: _createSlots, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, _createSlots({ _: 2 /* DYNAMIC */ }, [
    (a)
      ? {
        name: \"header\",
        fn: _withCtx(() => [
          _createTextVNode(\"x\")
        ]),
        key: \"0\"
      }
    : {
      name: \"header\",
      fn: _withCtx(() => [
        _createTextVNode(\"y\")
      ]),
      key: \"1\"
    }
  ]), 1024 /* DYNAMIC_SLOTS */))
}")
    );
}
