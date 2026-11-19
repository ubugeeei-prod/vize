use super::*;

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
