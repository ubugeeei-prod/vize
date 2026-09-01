use super::*;

#[test]
fn empty_div_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div></div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\"))
}"
    );
}

#[test]
fn div_with_text_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div>hello</div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, \"hello\"))
}"
    );
}

#[test]
fn nested_elements_match_the_shipped_snapshot() {
    assert_eq!(
        assembled("<div><span>hello</span></div>"),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", null, \"hello\")
  ]))
}"
    );
}

#[test]
fn emit_dom_source_agrees_with_emit_dom() {
    let allocator = Allocator::new();
    let via_source = emit_dom_source(&allocator, "<p>hi</p>").expect("emit");
    assert_eq!(assembled("<p>hi</p>"), via_source.assembled().as_str());
}

#[test]
fn empty_div_with_class_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div class="x"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { class: \"x\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn multiple_static_attrs_hoist_as_one_object() {
    assert_eq!(
        assembled(r#"<div id="app" class="container">static</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { id: \"app\", class: \"container\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1, \"static\"))
}"
    );
}

#[test]
fn hyphenated_attr_names_are_quoted() {
    assert_eq!(
        assembled(r#"<div data-id="1"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { \"data-id\": \"1\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn boolean_attr_emits_an_empty_string_value() {
    assert_eq!(
        assembled("<div disabled></div>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { disabled: \"\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _hoisted_1))
}"
    );
}

#[test]
fn nested_static_attrs_match_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div><span class="x">hello</span></div>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", { class: \"x\" }, \"hello\")
  ]))
}"
    );
}

#[test]
fn a_bound_class_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div :class="cls"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(cls)
  }, null, 2 /* CLASS */))
}"
    );
}

#[test]
fn bounded_string_class_concats_drop_the_legacy_class_patch_flag() {
    assert_eq!(
        assembled(r#"<div :class="'is-' + state + '-active'"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass('is-' + state + '-active')
  }))
}"
    );
}

#[test]
fn bounded_string_style_concats_drop_the_legacy_style_patch_flag() {
    assert_eq!(
        assembled(r#"<div :style="'width:' + size + 'px'"></div>"#),
        "\
const { normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    style: _normalizeStyle('width:' + size + 'px')
  }))
}"
    );
}

#[test]
fn bounded_string_element_attrs_drop_the_legacy_props_patch_flag() {
    assert_shipped_parity(r#"<a :href="'/#/' + lang + '/component/custom-theme'">{{ label }}</a>"#);
    assert_shipped_parity(
        r#"<span :title="'【' + site + '】' + name + ' 第' + index + '集'">{{ label }}</span>"#,
    );
}

#[test]
fn in_condition_element_attrs_keep_the_legacy_props_patch_flag() {
    assert_shipped_parity(
        r#"<span :title="'type' in value ? value.type : translate('schema.unknownType')">x</span>"#,
    );
}

#[test]
fn unparenthesized_in_conditionals_drop_legacy_class_and_style_patch_flags() {
    assert_eq!(
        assembled(r#"<div :class="'session' in row && row.session ? 'locked' : ''"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass('session' in row && row.session ? 'locked' : '')
  }))
}"
    );
    assert_eq!(
        assembled(r#"<div :style="'visible' in state ? 'display:block' : 'display:none'"></div>"#),
        "\
const { normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    style: _normalizeStyle('visible' in state ? 'display:block' : 'display:none')
  }))
}"
    );
}

#[test]
fn parenthesized_legacy_patchless_neighbors_keep_class_and_style_flags() {
    assert_eq!(
        assembled(
            r#"<div :class="('is-' + state + '-active')" :style="('width:' + size + 'px')"></div>"#,
        ),
        "\
const { normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(('is-' + state + '-active')),
    style: _normalizeStyle(('width:' + size + 'px'))
  }, null, 6 /* CLASS, STYLE */))
}"
    );
}

#[test]
fn class_plus_this_style_objects_skip_the_legacy_style_normalizer() {
    assert_eq!(
        assembled(r#"<div :class="c" :style="{ height: this.h }"></div>"#),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(c),
    style: { height: this.h }
  }, null, 6 /* CLASS, STYLE */))
}"
    );
}

#[test]
fn dynamic_style_objects_keep_the_legacy_style_normalizer() {
    assert_eq!(
        assembled(r#"<div :style="{ height: this.h, width: w }"></div>"#),
        "\
const { normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    style: _normalizeStyle({ height: this.h, width: w })
  }, null, 4 /* STYLE */))
}"
    );
}

#[test]
fn full_props_patch_flags_drop_class_and_style_bits() {
    assert_eq!(
        assembled(r#"<div :[foo]="bar" :class="c" :style="s"></div>"#),
        "\
const { normalizeProps: _normalizeProps, normalizeClass: _normalizeClass, normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({
    [_ctx.foo || \"\"]: bar,
    class: _normalizeClass(c),
    style: _normalizeStyle(s)
  }), null, 16 /* FULL_PROPS */))
}"
    );
    assert_eq!(
        assembled(r#"<div :class="c" @[name]="handler"></div>"#),
        "\
const { normalizeClass: _normalizeClass, toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass(c),
    [_toHandlerKey(_ctx.name)]: handler
  }, null, 16 /* FULL_PROPS */))
}"
    );
}
