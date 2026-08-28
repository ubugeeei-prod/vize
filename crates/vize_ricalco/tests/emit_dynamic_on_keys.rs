//! Dynamic-name `ui.on` emit pins (`@[event]`).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_ricalco::{EmitError, UnsupportedReason as Reason, emit_dom};

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

fn refused(source: &str) -> EmitError {
    with_transformed(source, |lowered, _, facts, _| {
        emit_dom(lowered, facts).expect_err("expected Unsupported")
    })
}

#[test]
fn a_dynamic_event_name_uses_to_handler_key() {
    assert_eq!(
        assembled(r#"<button @[event]="handler"></button>"#),
        "\
const { toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", { [_toHandlerKey(_ctx.event)]: handler }, null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_event_name_call_prefixes_the_callee() {
    assert_eq!(
        assembled(r#"<button @[eventOf()]="handler"></button>"#),
        "\
const { toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", { [_toHandlerKey(_ctx.eventOf())]: handler }, null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_event_name_keeps_scope_aliases_local() {
    assert_eq!(
        assembled(r#"<button v-for="item in items" @[item.event]="item.handler"></button>"#),
        "\
const { toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
    return (_openBlock(), _createElementBlock(\"button\", { [_toHandlerKey(item.event)]: item.handler }, null, 16 /* FULL_PROPS */))
  }), 256 /* UNKEYED_FRAGMENT */))
}"
    );
}

#[test]
fn a_dynamic_event_name_with_inline_handler_stays_full_props() {
    assert_eq!(
        assembled(r#"<button @[event]="handler($event)"></button>"#),
        "\
const { toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", {
    [_toHandlerKey(_ctx.event)]: $event => (handler($event))
  }, null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_event_name_with_event_modifiers_wraps_with_modifiers() {
    assert_eq!(
        assembled(r#"<button @[event].stop.prevent="handler"></button>"#),
        "\
const { withModifiers: _withModifiers, toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", {
    [_toHandlerKey(_ctx.event)]: _withModifiers(handler, [\"stop\",\"prevent\"])
  }, null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_event_name_with_key_modifiers_nests_keys_outside_modifiers() {
    assert_eq!(
        assembled(r#"<button @[event].enter.stop="handler"></button>"#),
        "\
const { withKeys: _withKeys, withModifiers: _withModifiers, toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", {
    [_toHandlerKey(_ctx.event)]: _withKeys(_withModifiers(handler, [\"stop\"]), [\"enter\"])
  }, null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_event_name_with_option_modifiers_keeps_the_computed_key() {
    assert_eq!(
        assembled(r#"<button @[event].once.capture.passive="handler"></button>"#),
        "\
const { toHandlerKey: _toHandlerKey, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", { [_toHandlerKey(_ctx.event)]: handler }, null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn a_dynamic_event_name_must_be_js() {
    assert_eq!(
        refused(r#"<button @[event.]="handler"></button>"#).reason(),
        Some(Reason::OnNameNotJs)
    );
}
