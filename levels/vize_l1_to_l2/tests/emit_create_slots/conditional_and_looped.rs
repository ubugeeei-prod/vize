use super::*;

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
fn slotted_component_concatenated_string_props_keep_dynamic_prop_list() {
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
