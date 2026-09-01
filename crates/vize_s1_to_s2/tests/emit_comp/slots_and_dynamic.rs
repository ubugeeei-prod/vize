use super::*;

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
fn static_ref_on_dynamic_slots_sets_need_patch() {
    assert_shipped_parity(
        r#"<Foo ref="date-picker"><template v-for="slotName in slotList" #[slotName]="args" :key="slotName"><slot :name="slotName as keyof RootSlots" v-bind="args" /></template></Foo>"#,
    );
}

#[test]
fn implicit_default_text_runs_keep_legacy_split_vnodes() {
    assert_shipped_parity(r#"<Foo>{{ a }}-{{ b }}</Foo>"#);
}

#[test]
fn component_root_v_slot_merges_text_runs_like_template_slots() {
    assert_shipped_parity(
        r#"<Foo #="{ year, month, date }">{{ year }}-{{ month }}-{{ date }}</Foo>"#,
    );
}

#[test]
fn v_for_component_root_v_slot_key_uses_legacy_multiline_props() {
    assert_shipped_parity(
        r#"<Foo><Bar v-for="n in 5" :key="n" v-slot="{ isSelected, toggle }"><Baz :color="isSelected ? 'primary' : undefined" class="ma-2" /></Bar></Foo>"#,
    );
}

#[test]
fn component_static_props_hoist_after_inline_root_props() {
    assert_eq!(
        assembled(r#"<div id="root"><Foo id="x">hello</Foo></div>"#),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

const _hoisted_1 = { id: \"x\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createElementBlock(\"div\", { id: \"root\" }, [
    _createVNode(_component_Foo, _hoisted_1, {
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
fn a_nested_text_slot_inside_v_for_is_dynamic() {
    assert_eq!(
        assembled(r#"<div v-for="i in n"><Foo>hello</Foo></div>"#),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createTextVNode: _createTextVNode, renderList: _renderList, withCtx: _withCtx } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(n, (i) => {
    return (_openBlock(), _createElementBlock(\"div\", null, [
      _createVNode(_component_Foo, null, {
        default: _withCtx(() => [
          _createTextVNode(\"hello\")
        ]),
        _: 2 /* DYNAMIC */
      }, 1024 /* DYNAMIC_SLOTS */)
    ]))
  }), 256 /* UNKEYED_FRAGMENT */))
}")
    );
}

#[test]
fn whitespace_only_children_emit_no_slot() {
    assert_eq!(assembled("<Foo>  </Foo>"), assembled("<Foo />"));
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
fn a_dynamic_is_keeps_authored_trailing_padding() {
    assert_shipped_parity(
        r#"<component :is="ok
  ? Foo
  : Bar " />"#,
    );
}

#[test]
fn a_static_is_slot_hoists_props_without_is() {
    assert_eq!(
        assembled(r#"<component is="Foo" id="a">hello</component>"#),
        pin("\
const { resolveDynamicComponent: _resolveDynamicComponent, openBlock: _openBlock, createBlock: _createBlock, createTextVNode: _createTextVNode, withCtx: _withCtx } = Vue

const _hoisted_1 = { id: \"a\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(\"Foo\"), _hoisted_1, {
    default: _withCtx(() => [
      _createTextVNode(\"hello\")
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn a_component_object_on_uses_to_handlers() {
    assert_eq!(
        assembled(r#"<Foo v-on="handlers" />"#),
        pin("\
const { resolveComponent: _resolveComponent, toHandlers: _toHandlers, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, _toHandlers(handlers, true), null, 16 /* FULL_PROPS */))
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
