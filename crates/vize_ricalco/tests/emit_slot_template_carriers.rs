//! Malformed slot-template carriers that are not the owning component's
//! direct slot group emit as inline `<template>` nodes, matching the
//! shipped lane's fallback path.

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
fn a_nested_slot_template_interpolation_emits_inline_expression() {
    assert_eq!(
        assembled(r#"<Foo><template #header><template #inner>{{ msg }}</template></template></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, toDisplayString: _toDisplayString, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    header: _withCtx(() => [
      _toDisplayString(msg)
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_nested_slot_template_multiple_children_stay_one_inline_array() {
    assert_eq!(
        assembled(
            r#"<Foo><template #header><template #inner><b></b><i></i></template></template></Foo>"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"b\")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode(\"i\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    header: _withCtx(() => [
      [_hoisted_1, _hoisted_2]
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_stray_slot_template_inside_native_children_uses_inline_template_emit() {
    assert_eq!(
        assembled(r#"<div><template #inner>{{ msg }}</template></div>"#),
        pin("\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _toDisplayString(msg)
  ]))
}")
    );
}

#[test]
fn create_slots_nested_slot_template_uses_inline_template_emit() {
    assert_eq!(
        assembled(
            r#"<Foo><template #header v-if="ok"><template #inner>{{ msg }}</template></template></Foo>"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, toDisplayString: _toDisplayString, openBlock: _openBlock, createBlock: _createBlock, createSlots: _createSlots, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, _createSlots({ _: 2 /* DYNAMIC */ }, [
    (ok)
      ? {
        name: \"header\",
        fn: _withCtx(() => [
          _toDisplayString(msg)
        ]),
        key: \"0\"
      }
    : undefined
  ]), 1024 /* DYNAMIC_SLOTS */))
}")
    );
}
