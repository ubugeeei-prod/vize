use super::*;

#[test]
fn typed_arrow_component_model_keeps_nested_assignment_padding() {
    assert_eq!(
        assembled(
            r#"<Foo v-model="
  options[options.findIndex((e: any) => e.id === activeId)]
" />"#,
        ),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    modelValue: options[options.findIndex((e: any) => e.id === activeId)],
    \"onUpdate:modelValue\": $event => ($event => ($event => ((
  options[options.findIndex((e: any) => e.id === activeId)]
) = $event)))
  }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))
}")
    );
}

#[test]
fn a_named_component_model_uses_the_argument() {
    assert_eq!(
        assembled(r#"<Foo v-model:title="pageTitle" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    title: pageTitle,
    \"onUpdate:title\": $event => ((pageTitle) = $event)
  }, null, 8 /* PROPS */, [\"title\", \"onUpdate:title\"]))
}")
    );
}

#[test]
fn a_kebab_component_model_camelizes_the_update_listener() {
    assert_eq!(
        assembled(r#"<Foo v-model:auto-send="autoSendEnabled" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    \"auto-send\": autoSendEnabled,
    \"onUpdate:autoSend\": $event => ((autoSendEnabled) = $event)
  }, null, 8 /* PROPS */, [\"auto-send\", \"onUpdate:autoSend\"]))
}")
    );
}

#[test]
fn a_dynamic_component_model_uses_computed_props() {
    assert_eq!(
        assembled(r#"<Foo v-model:[field]="msg" />"#),
        pin("\
const { resolveComponent: _resolveComponent, normalizeProps: _normalizeProps, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, _normalizeProps({ [field]: msg,
  [\"onUpdate:\" + field]: $event => ((msg) = $event) }), null, 16 /* FULL_PROPS */))
}")
    );
}

#[test]
fn a_dynamic_component_model_modifier_uses_a_computed_modifiers_key() {
    assert_eq!(
        assembled(r#"<Foo v-model:[field].trim="msg" />"#),
        pin("\
const { resolveComponent: _resolveComponent, normalizeProps: _normalizeProps, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, _normalizeProps({ [field]: msg,
  [\"onUpdate:\" + field]: $event => ((msg) = $event),
  [field + \"Modifiers\"]: { trim: true } }), null, 16 /* FULL_PROPS */))
}")
    );
}

#[test]
fn component_modifiers_are_constant_model_modifiers() {
    assert_eq!(
        assembled(r#"<Foo v-model.lazy.trim="msg" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    modelValue: msg,
    \"onUpdate:modelValue\": $event => ((msg) = $event),
    modelModifiers: { lazy: true, trim: true }
  }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))
}")
    );
}

#[test]
fn a_plain_div_model_emits_the_handler_without_directives() {
    assert_eq!(
        assembled(r#"<div v-model="msg"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    \"onUpdate:modelValue\": $event => ((msg) = $event)
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"]))
}"
    );
}

#[test]
fn a_custom_directive_beside_v_model_merges_into_one_wrap() {
    assert_eq!(
        assembled(r#"<input v-model="msg" v-example>"#),
        pin("\
const { resolveDirective: _resolveDirective, vModelText: _vModelText, withDirectives: _withDirectives, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _directive_example = _resolveDirective(\"example\")

  return _withDirectives((_openBlock(), _createElementBlock(\"input\", {
    \"onUpdate:modelValue\": $event => ((msg) = $event)
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"])), [
    [_vModelText, msg],
    [_directive_example]
  ])
}")
    );
}

#[test]
fn a_component_model_merges_an_update_listener_in_source_order() {
    assert_eq!(
        assembled(r#"<Foo v-model="x" @update:modelValue="h" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    modelValue: x,
    \"onUpdate:modelValue\": [$event => ((x) = $event), h]
  }, null, 8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]))
}")
    );
}

#[test]
fn a_listener_before_v_model_keeps_source_order() {
    assert_eq!(
        assembled(r#"<Foo @update:modelValue="h" v-model="x" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, {
    \"onUpdate:modelValue\": [h, $event => ((x) = $event)],
    modelValue: x
  }, null, 8 /* PROPS */, [\"onUpdate:modelValue\", \"modelValue\"]))
}")
    );
}

#[test]
fn a_missing_expression_is_a_diagnostic_refusal() {
    assert_eq!(refused(r#"<input v-model>"#), EmitError::Diagnostics);
}
