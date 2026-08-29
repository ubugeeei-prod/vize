//! `v-slots` spread emit pins (children argument / `...expr`).

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
fn only_forwarded_slots_are_the_children_argument() {
    assert_eq!(
        assembled(r#"<Comp v-slots="slots" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Comp = _resolveComponent(\"Comp\")

  return (_openBlock(), _createBlock(_component_Comp, null, slots, 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn a_spread_closes_authored_slots_without_a_stability_flag() {
    assert_eq!(
        assembled(r#"<Comp v-slots="slots"><span></span></Comp>"#),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Comp = _resolveComponent(\"Comp\")

  return (_openBlock(), _createBlock(_component_Comp, null, {
    default: _withCtx(() => [
      _hoisted_1
    ]),
    ...slots
  }, 1024 /* DYNAMIC_SLOTS */))
}")
    );
}

#[test]
fn static_props_stay_inline_beside_a_slots_spread() {
    assert_eq!(
        assembled(r#"<Comp id="x" v-slots="slots" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Comp = _resolveComponent(\"Comp\")

  return (_openBlock(), _createBlock(_component_Comp, { id: \"x\" }, slots, 1024 /* DYNAMIC_SLOTS */))
}")
    );
}
