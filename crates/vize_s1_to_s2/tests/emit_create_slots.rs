//! `createSlots` emit pins (`v-if` / `v-for` slot templates).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom;

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

fn shipped(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template(&allocator, source);
    let blocking: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_compatibility_notice())
        .collect();
    assert!(blocking.is_empty(), "{source:?}: {blocking:?}");
    format!("{}\n{}", old.preamble, old.code)
}

fn assert_shipped_parity(source: &str) {
    assert_eq!(assembled(source), shipped(source), "{source}");
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
fn conditional_slot_template_preserves_authored_condition_padding() {
    assert_shipped_parity(
        r#"<Foo><template #header v-if="
  ok &&
  ready
">x</template></Foo>"#,
    );
}

#[test]
fn conditional_scoped_slot_template_preserves_authored_param_padding() {
    assert_shipped_parity(
        r#"<Foo><template #header="slotProps " v-if="ok"><slot v-bind="slotProps" /></template></Foo>"#,
    );
}

#[test]
fn slotted_component_legacy_patchless_concat_props_leave_dynamic_prop_list() {
    assert_shipped_parity(r#"<Foo><Bar :foo="'a' + i + 'b'"><Baz /></Bar></Foo>"#);
    assert_shipped_parity(
        r#"<Foo><Bar v-for="(item, index) in items" :key="item.key" :label="'Label' + index" :prop="'items.' + index + '.value'" :rules="{ required: true }"><Baz /></Bar></Foo>"#,
    );
}

#[test]
fn static_attrs_on_conditional_slot_templates_are_elided() {
    assert_eq!(
        assembled(r#"<Foo><template #header id="x" v-if="ok">x</template></Foo>"#),
        assembled(r#"<Foo><template #header v-if="ok">x</template></Foo>"#)
    );
}

#[test]
fn inert_builtin_bindings_on_conditional_slot_templates_are_elided() {
    let plain = assembled(r#"<Foo><template #header v-if="ok">x</template></Foo>"#);
    assert_eq!(
        assembled(r#"<Foo><template #header v-if="ok" v-once>x</template></Foo>"#),
        plain
    );
    assert_eq!(
        assembled(r#"<Foo><template #header v-if="ok" v-memo="[ok]">x</template></Foo>"#),
        plain
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
fn static_attrs_on_looped_slot_templates_are_elided() {
    assert_eq!(
        assembled(r#"<Foo><template v-for="i in n" #header id="x">x</template></Foo>"#),
        assembled(r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#)
    );
}

#[test]
fn key_bindings_on_looped_slot_templates_are_elided() {
    assert_eq!(
        assembled(r#"<Foo><template v-for="i in n" #header :key="i">x</template></Foo>"#),
        assembled(r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#)
    );
}

#[test]
fn key_bindings_on_dynamic_forwarding_slot_templates_are_elided() {
    assert_eq!(
        assembled(
            r#"<Foo><template v-for="(_, name) in $slots" :key="name" #[name]="slotData"><slot :name="name" v-bind="slotData || {}" /></template></Foo>"#
        ),
        assembled(
            r#"<Foo><template v-for="(_, name) in $slots" #[name]="slotData"><slot :name="name" v-bind="slotData || {}" /></template></Foo>"#
        )
    );
}

#[test]
fn inert_builtin_bindings_on_looped_slot_templates_are_elided() {
    let plain = assembled(r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#);
    assert_eq!(
        assembled(r#"<Foo><template v-for="i in n" #header v-once>x</template></Foo>"#),
        plain
    );
    assert_eq!(
        assembled(r#"<Foo><template v-for="i in n" #header v-memo="[i]">x</template></Foo>"#),
        plain
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
fn non_slot_if_siblings_stay_in_create_slots_dynamic_entries_as_undefined() {
    assert_shipped_parity(
        r#"<Foo><Bar v-if="bar" /><Baz v-if="baz" /><template v-for="(_, name) in slots" #[name]="slotData"><slot :name="name" v-bind="slotData" /></template></Foo>"#,
    );
}

#[test]
fn create_slots_default_branch_keys_are_allocated_before_named_entries() {
    assert_shipped_parity(
        r#"<Foo><template #body><Bar v-if="bodyA" /><Baz v-if="bodyB" /></template><Qux v-if="mainA" /><Quux v-else-if="mainB" /><template v-if="footer" #footer><Footer /></template></Foo>"#,
    );
}

#[test]
fn dynamic_slot_template_branch_keys_do_not_leak_to_parent_slots() {
    assert_shipped_parity(
        r#"<Comp><template #a><Inner><template v-if="x" #input><div /></template></Inner></template><div v-if="y" /></Comp>"#,
    );
    assert_shipped_parity(
        r#"<Comp><template #a><Inner><template #input><div v-if="x" /></template></Inner></template><div v-if="y" /></Comp>"#,
    );
    assert_shipped_parity(
        r#"<Comp><template #a><Inner><div v-if="x" /></Inner></template><div v-if="y" /></Comp>"#,
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
fn unwrapped_if_with_nested_slot_keeps_sibling_vnodes() {
    assert_eq!(
        assembled(
            r#"<Foo><template v-if="ok"><span>x</span><template #header>y</template></template></Foo>"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\", null, \"x\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      (ok)
        ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
          _hoisted_1,
          _createTextVNode(\"y\")
        ], 64 /* STABLE_FRAGMENT */))
        : _createCommentVNode(\"v-if\", true)
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn unwrapped_for_with_nested_slot_keeps_sibling_vnodes() {
    assert_eq!(
        assembled(
            r#"<Foo><template v-for="i in n"><span>x</span><template #header>y</template></template></Foo>"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createTextVNode: _createTextVNode, renderList: _renderList, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\", null, \"x\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
        return (_openBlock(), _createElementBlock(_Fragment, null, [
          _hoisted_1,
          _createTextVNode(\"y\")
        ], 64 /* STABLE_FRAGMENT */))
      }), 256 /* UNKEYED_FRAGMENT */))
    ]),
    _: 1 /* STABLE */
  }))
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
