use super::*;

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
fn non_slot_for_slot_outlet_siblings_leave_empty_create_slots_entries() {
    assert_shipped_parity(
        r#"<Foo><template v-for="slotKey in keys"><slot :name="slotKey" /></template><template v-for="(component, slot) in schemaSlots" #[slot]="slotProps"><slot :name="slot" v-bind="slotProps"><component :is="component" /></slot></template></Foo>"#,
    );
}

#[test]
fn component_slots_inside_dynamic_v_for_entries_stay_stable() {
    assert_shipped_parity(
        r#"<RecipeList><template v-for="(recipe, index) in recipes" #[`actions-${recipe.id}`] :key="'item-actions-decrease' + recipe.id"><Action><Button v-if="recipe" :disabled="off" @click.prevent="remove(recipe.id!)"><Icon color="grey">{{ icon }}</Icon></Button></Action><div>{{ quantities[index].value }}</div></template></RecipeList>"#,
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
