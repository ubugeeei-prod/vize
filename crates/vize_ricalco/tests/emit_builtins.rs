//! Vue builtin emit pins (`Teleport` / `KeepAlive` / `Transition` /
//! `Suspense` / `TransitionGroup` / `BaseTransition`).

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

fn pin(visual: &str) -> String {
    visual.replace(")\n\n  return", ")\n  \n  return")
}

#[test]
fn a_bare_teleport_uses_the_helper() {
    assert_eq!(
        assembled("<Teleport />"),
        pin("\
const { openBlock: _openBlock, createBlock: _createBlock, Teleport: _Teleport } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock(_Teleport))
}")
    );
}

#[test]
fn teleport_children_are_an_array_with_hoisted_props() {
    assert_eq!(
        assembled(r##"<Teleport to="#app"><span></span></Teleport>"##),
        pin("\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, Teleport: _Teleport } = Vue

const _hoisted_1 = { to: \"#app\" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock(_Teleport, _hoisted_1, [
    _hoisted_2
  ]))
}")
    );
}

#[test]
fn teleport_interpolation_keeps_the_text_flag() {
    assert_eq!(
        assembled(r##"<Teleport to="#app">hello {{ msg }}</Teleport>"##),
        pin("\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, Teleport: _Teleport } = Vue

const _hoisted_1 = { to: \"#app\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock(_Teleport, _hoisted_1, [
    _createTextVNode(\"hello \"),
    _toDisplayString(msg)
  ], 1 /* TEXT */))
}")
    );
}

#[test]
fn a_nested_teleport_stays_a_block() {
    assert_eq!(
        assembled(r##"<div><Teleport to="#app"><span></span></Teleport></div>"##),
        pin("\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createBlock: _createBlock, Teleport: _Teleport } = Vue

const _hoisted_1 = { to: \"#app\" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    (_openBlock(), _createBlock(_Teleport, _hoisted_1, [
      _hoisted_2
    ]))
  ]))
}")
    );
}

#[test]
fn keepalive_always_flags_dynamic_slots() {
    assert_eq!(
        assembled("<KeepAlive><Foo /></KeepAlive>"),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createBlock: _createBlock, KeepAlive: _KeepAlive } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_KeepAlive, null, [
    _createVNode(_component_Foo)
  ], 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn empty_keepalive_emits_null_children() {
    assert_eq!(
        assembled("<KeepAlive />"),
        pin("\
const { openBlock: _openBlock, createBlock: _createBlock, KeepAlive: _KeepAlive } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock(_KeepAlive, null, null, 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn transition_children_are_a_slot_object() {
    assert_eq!(
        assembled("<Transition><div></div></Transition>"),
        pin("\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, Transition: _Transition, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"div\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock(_Transition, null, {
    default: _withCtx(() => [
      _hoisted_1
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_nested_transition_uses_create_vnode() {
    assert_eq!(
        assembled("<div><Transition><span></span></Transition></div>"),
        pin("\
const { createElementVNode: _createElementVNode, createVNode: _createVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Transition: _Transition, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createVNode(_Transition, null, {
      default: _withCtx(() => [
        _hoisted_1
      ]),
      _: 1 /* STABLE */
    })
  ]))
}")
    );
}

#[test]
fn keepalive_inside_a_slot_follows_with_ctx() {
    assert_eq!(
        assembled("<Foo><KeepAlive><Bar /></KeepAlive></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx, KeepAlive: _KeepAlive } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Bar = _resolveComponent(\"Bar\")
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      (_openBlock(), _createBlock(_KeepAlive, null, [
        _createVNode(_component_Bar)
      ], 1024 /* DYNAMIC_SLOTS */))
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn kebab_teleport_still_imports_pascal_helper() {
    assert_eq!(
        assembled(r##"<teleport to="#app" />"##),
        assembled(r##"<Teleport to="#app" />"##)
    );
}
