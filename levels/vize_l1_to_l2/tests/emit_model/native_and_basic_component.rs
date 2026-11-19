use super::*;

#[test]
fn a_native_text_input_wraps_with_v_model_text() {
    assert_eq!(
        assembled(r#"<input v-model="msg">"#),
        "\
const { vModelText: _vModelText, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _withDirectives((_openBlock(), _createElementBlock(\"input\", {
    \"onUpdate:modelValue\": $event => ((msg) = $event)
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"])), [
    [_vModelText, msg]
  ])
}"
    );
}

#[test]
fn an_empty_select_selects_v_model_select() {
    assert_eq!(
        assembled(r#"<select v-model="msg"></select>"#),
        "\
const { vModelSelect: _vModelSelect, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _withDirectives((_openBlock(), _createElementBlock(\"select\", {
    \"onUpdate:modelValue\": $event => ((msg) = $event)
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"])), [
    [_vModelSelect, msg]
  ])
}"
    );
}

#[test]
fn a_checkbox_selects_v_model_checkbox() {
    assert_eq!(
        assembled(r#"<input type="checkbox" v-model="ok">"#),
        "\
const { vModelCheckbox: _vModelCheckbox, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _withDirectives((_openBlock(), _createElementBlock(\"input\", {
    type: \"checkbox\",
    \"onUpdate:modelValue\": $event => ((ok) = $event)
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"])), [
    [_vModelCheckbox, ok]
  ])
}"
    );
}

#[test]
fn lazy_trim_number_emit_the_modifier_object() {
    assert_eq!(
        assembled(r#"<input v-model.lazy="msg">"#),
        "\
const { vModelText: _vModelText, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _withDirectives((_openBlock(), _createElementBlock(\"input\", {
    \"onUpdate:modelValue\": $event => ((msg) = $event)
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"])), [
    [
      _vModelText,
      msg,
      void 0,
      { lazy: true }
    ]
  ])
}"
    );
    assert_eq!(
        assembled(r#"<input v-model.trim.number="msg">"#),
        "\
const { vModelText: _vModelText, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _withDirectives((_openBlock(), _createElementBlock(\"input\", {
    \"onUpdate:modelValue\": $event => ((msg) = $event)
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"])), [
    [
      _vModelText,
      msg,
      void 0,
      {
        trim: true,
        number: true
      }
    ]
  ])
}"
    );
}

#[test]
fn a_component_model_emits_product_props() {
    assert_eq!(
        assembled(r#"<Foo v-model="msg" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    modelValue: msg,
    \"onUpdate:modelValue\": $event => ((msg) = $event)
  }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))
}")
    );
}

#[test]
fn component_model_assignment_keeps_authored_multiline_expression_padding() {
    assert_eq!(
        assembled(
            r#"<Foo v-model="
  form.value
" />"#
        ),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    modelValue: form.value,
    \"onUpdate:modelValue\": $event => ((
  form.value
) = $event)
  }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))
}")
    );
}

#[test]
fn conditional_component_model_uses_the_legacy_nested_assignment_callback() {
    assert_eq!(
        assembled(r#"<Foo v-model="multiple ? presentText : inputValue" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    modelValue: multiple ? presentText : inputValue,
    \"onUpdate:modelValue\": $event => ($event => ($event => ((multiple ? presentText : inputValue) = $event)))
  }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))
}")
    );
}

#[test]
fn typed_arrow_component_model_uses_the_legacy_nested_assignment_callback() {
    assert_eq!(
        assembled(r#"<Foo v-model="options[options.findIndex((e: any) => e.id === activeId)]" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    modelValue: options[options.findIndex((e: any) => e.id === activeId)],
    \"onUpdate:modelValue\": $event => ($event => ($event => ((options[options.findIndex((e: any) => e.id === activeId)]) = $event)))
  }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))
}")
    );
}
