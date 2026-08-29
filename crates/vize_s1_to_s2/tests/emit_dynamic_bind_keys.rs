//! P2-11: dynamic-argument `v-bind` keys in the S2 DOM emitter.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s1_to_s2::emit_dom;

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

#[test]
fn a_dynamic_bind_key_uses_a_computed_prop_and_full_props_patch() {
    assert_eq!(
        assembled(r#"<div :[key]="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [_ctx.key || \"\"]: value }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_bind_template_literal_uses_the_shipped_key_expression() {
    assert_eq!(
        assembled(r#"<div :[`data-${id}`]="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [(`data-${_ctx.id}`) || \"\"]: value }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn dynamic_bind_template_literal_prefixes_each_runtime_reference() {
    assert_eq!(
        assembled(r#"<div :[`data-${prefix+suffix}`]="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [(`data-${_ctx.prefix+_ctx.suffix}`) || \"\"]: value }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn dynamic_bind_template_literal_keeps_global_names_unprefixed() {
    assert_eq!(
        assembled(r#"<div :[`data-${Math.random()}`]="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [(`data-${Math.random()}`) || \"\"]: value }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_bind_template_literal_keeps_loop_aliases_local() {
    assert_eq!(
        assembled(r#"<div v-for="item in items" :[`data-${item.id}`]="item.value"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
    return (_openBlock(), _createElementBlock(\"div\", { [(`data-${item.id}`) || \"\"]: item.value }, null, 16 /* FULL_PROPS */))
  }), 256 /* UNKEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_dynamic_bind_key_inside_v_for_skips_the_normalize_props_wrapper() {
    assert_eq!(
        assembled(r#"<div v-for="item in items" :[name]="item.value"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
    return (_openBlock(), _createElementBlock(\"div\", { [_ctx.name || \"\"]: item.value }, null, 16 /* FULL_PROPS */))
  }), 256 /* UNKEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_dynamic_camel_bind_key_uses_the_runtime_camelize_helper() {
    assert_eq!(
        assembled(r#"<div :[key].camel="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, camelize: _camelize, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [_camelize(_ctx.key || \"\")]: value }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn dynamic_bind_key_modifiers_compose_in_vue_order() {
    assert_eq!(
        assembled(r#"<div :[key].camel.prop="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, camelize: _camelize, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [`.${_camelize(_ctx.key || \"\")}`]: value }), null, 48 /* FULL_PROPS, NEED_HYDRATION */))
}"
    );
    assert_eq!(
        assembled(r#"<div :[key].camel.attr="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, camelize: _camelize, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [`^${_camelize(_ctx.key || \"\")}`]: value }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn dynamic_prop_and_attr_modifiers_prefix_the_computed_key() {
    assert_eq!(
        assembled(r#"<div :[key].prop="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [`.${_ctx.key || \"\"}`]: value }), null, 48 /* FULL_PROPS, NEED_HYDRATION */))
}"
    );
    assert_eq!(
        assembled(r#"<div :[key].attr="value"></div>"#),
        "\
const { normalizeProps: _normalizeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ [`^${_ctx.key || \"\"}`]: value }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_bind_key_beside_an_object_spread_uses_merge_props() {
    assert_eq!(
        assembled(r#"<div v-bind="bag" :[key]="value"></div>"#),
        "\
const { mergeProps: _mergeProps, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _mergeProps(bag, { [_ctx.key || \"\"]: value }), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn key_spelled_as_a_dynamic_argument_still_feeds_branch_and_loop_keys() {
    assert_eq!(
        assembled(r#"<div v-if="ok" :[key]="value"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: value }))
    : _createCommentVNode(\"v-if\", true)
}"
    );
    assert_eq!(
        assembled(r#"<div v-for="item in items" :[key]="item.value"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
    return (_openBlock(), _createElementBlock(\"div\", { key: item.value }, null, 16 /* FULL_PROPS */))
  }), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn slot_outlet_dynamic_bind_keys_do_not_normalize_the_slot_props() {
    assert_eq!(
        assembled(r#"<slot :[key]="value" />"#),
        "\
const { renderSlot: _renderSlot } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"default\", { [_ctx.key || \"\"]: value })
}"
    );
}
