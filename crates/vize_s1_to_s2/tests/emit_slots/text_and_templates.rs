use super::*;

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
