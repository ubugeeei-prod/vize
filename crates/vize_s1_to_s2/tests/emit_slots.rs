//! Implicit default-slot emit pins (`withCtx` / `_` / unused props hoist /
//! hoisted static element children).

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
fn conditional_component_siblings_preserve_authored_space_in_default_slot() {
    assert_shipped_parity(
        r#"<Button><IconCheck v-if="copied" /> <IconCopy v-else /> Copy Page</Button>"#,
    );
}

#[test]
fn conditional_component_slot_child_hoists_static_props() {
    assert_shipped_parity(
        r#"<Foo><template #default="{ item }"><template v-if="ok"><i18n-t keypath="x"><span /></i18n-t></template><i18n-t v-else keypath="y"><span /></i18n-t></template></Foo>"#,
    );
}

#[test]
fn a_text_default_slot_uses_with_ctx() {
    assert_eq!(
        assembled("<Foo>hello</Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _createTextVNode(\"hello\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn an_interpolation_default_slot_flags_text() {
    assert_eq!(
        assembled("<Foo>{{ msg }}</Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, toDisplayString: _toDisplayString, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _createTextVNode(_toDisplayString(msg), 1 /* TEXT */)
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn mixed_text_and_interpolation_are_separate_vnodes() {
    assert_eq!(
        assembled("<Foo>hello {{ msg }}</Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, toDisplayString: _toDisplayString, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _createTextVNode(\"hello \"),
      _createTextVNode(_toDisplayString(msg), 1 /* TEXT */)
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_nested_text_slot_uses_create_vnode() {
    assert_eq!(
        assembled("<div><Foo>hello</Foo></div>"),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createVNode(_component_Foo, null, {
      default: _withCtx(() => [
        _createTextVNode(\"hello\")
      ]),
      _: 1 /* STABLE */
    })
  ]))
}")
    );
}

#[test]
fn a_v_for_item_text_slot_is_dynamic() {
    assert_eq!(
        assembled(r#"<Foo v-for="item in list">hello</Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createTextVNode: _createTextVNode, renderList: _renderList, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(list, (item) => {
    return (_openBlock(), _createBlock(_component_Foo, null, {
      default: _withCtx(() => [
        _createTextVNode(\"hello\")
      ]),
      _: 2 /* DYNAMIC */
    }, 1024 /* DYNAMIC_SLOTS */))
  }), 256 /* UNKEYED_FRAGMENT */))
}")
    );
}

#[test]
fn static_attrs_on_a_slotted_component_use_their_hoist() {
    assert_eq!(
        assembled(r#"<Foo id="x">hello</Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

const _hoisted_1 = { id: \"x\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, _hoisted_1, {
    default: _withCtx(() => [
      _createTextVNode(\"hello\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn static_attrs_on_named_slot_templates_are_elided() {
    assert_eq!(
        assembled(r#"<Foo><template #header id="x">x</template></Foo>"#),
        assembled(r#"<Foo><template #header>x</template></Foo>"#)
    );
}

#[test]
fn inert_builtin_bindings_on_named_slot_templates_are_elided() {
    let plain = assembled(r#"<Foo><template #header>x</template></Foo>"#);
    assert_eq!(
        assembled(r#"<Foo><template #header v-once>x</template></Foo>"#),
        plain
    );
    assert_eq!(
        assembled(r#"<Foo><template #header v-memo="[ok]">x</template></Foo>"#),
        plain
    );
}

#[test]
fn a_bare_template_default_slot_child_is_hoisted() {
    assert_eq!(
        assembled("<Foo><template>x</template></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"template\", null, \"x\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _hoisted_1
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn whitespace_only_children_emit_no_slot() {
    assert_eq!(assembled("<Foo>  </Foo>"), assembled("<Foo />"));
}

#[test]
fn a_static_element_slot_child_hoists() {
    assert_eq!(
        assembled("<Foo><span></span></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _hoisted_1
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_nested_component_slot_child_uses_create_vnode() {
    assert_eq!(
        assembled("<Foo><Bar /></Foo>"),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Bar = _resolveComponent(\"Bar\")
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _createVNode(_component_Bar)
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_dynamic_element_slot_child_stays_inline() {
    assert_eq!(
        assembled(r#"<Foo><span :id="x"></span></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _createElementVNode(\"span\", { id: x }, null, 8 /* PROPS */, [\"id\"])
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn component_props_and_a_static_span_use_two_hoists() {
    assert_eq!(
        assembled(r#"<Foo class="x"><span></span></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, withCtx: _withCtx } = Vue

const _hoisted_1 = { class: \"x\" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, _hoisted_1, {
    default: _withCtx(() => [
      _hoisted_2
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_v_for_item_element_slot_is_dynamic() {
    assert_eq!(
        assembled(r#"<Foo v-for="i in n"><span></span></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
    return (_openBlock(), _createBlock(_component_Foo, null, {
      default: _withCtx(() => [
        _hoisted_1
      ]),
      _: 2 /* DYNAMIC */
    }, 1024 /* DYNAMIC_SLOTS */))
  }), 256 /* UNKEYED_FRAGMENT */))
}")
    );
}

#[test]
fn a_nested_v_for_component_omits_empty_props() {
    assert_eq!(
        assembled(r#"<Foo><Bar v-for="i in n" /></Foo>"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Bar = _resolveComponent(\"Bar\")
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
        return (_openBlock(), _createBlock(_component_Bar))
      }), 256 /* UNKEYED_FRAGMENT */))
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}
