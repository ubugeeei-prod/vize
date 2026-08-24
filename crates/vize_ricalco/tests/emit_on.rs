//! Static-name `ui.on` emit pins, including object `v-on` (`toHandlers`).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_ricalco::{EmitError, emit_dom};

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
fn a_click_handler_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div @click="handler"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { onClick: handler }, null, 8 /* PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn a_keyup_handler_sets_need_hydration() {
    assert_eq!(
        assembled(r#"<div @keyup="handler"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { onKeyup: handler }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onKeyup\"]))
}"
    );
}

#[test]
fn an_inline_click_wraps_as_an_arrow() {
    assert_eq!(
        assembled(r#"<div @click="count++"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: $event => (count++)
  }, null, 8 /* PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn a_click_stop_wraps_with_modifiers() {
    assert_eq!(
        assembled(r#"<div @click.stop="handler"></div>"#),
        "\
const { withModifiers: _withModifiers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: _withModifiers(handler, [\"stop\"])
  }, null, 8 /* PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn a_click_prevent_stop_keeps_author_order() {
    assert_eq!(
        assembled(r#"<div @click.prevent.stop="handler"></div>"#),
        "\
const { withModifiers: _withModifiers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: _withModifiers(handler, [\"prevent\",\"stop\"])
  }, null, 8 /* PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn a_click_capture_suffixes_the_event_key() {
    assert_eq!(
        assembled(r#"<div @click.capture="handler"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { onClickCapture: handler }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onClickCapture\"]))
}"
    );
}

#[test]
fn a_click_right_remaps_to_contextmenu() {
    assert_eq!(
        assembled(r#"<div @click.right="handler"></div>"#),
        "\
const { withModifiers: _withModifiers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onContextmenu: _withModifiers(handler, [\"right\"])
  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onContextmenu\"]))
}"
    );
}

#[test]
fn a_keyup_enter_wraps_with_keys() {
    assert_eq!(
        assembled(r#"<div @keyup.enter="handler"></div>"#),
        "\
const { withKeys: _withKeys, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onKeyup: _withKeys(handler, [\"enter\"])
  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onKeyup\"]))
}"
    );
}

#[test]
fn a_keyup_enter_stop_nests_keys_outside_modifiers() {
    assert_eq!(
        assembled(r#"<div @keyup.enter.stop="handler"></div>"#),
        "\
const { withKeys: _withKeys, withModifiers: _withModifiers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onKeyup: _withKeys(_withModifiers(handler, [\"stop\"]), [\"enter\"])
  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onKeyup\"]))
}"
    );
}

#[test]
fn a_click_native_is_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<div @click.native="handler"></div>"#),
        EmitError::Unsupported
    );
}

#[test]
fn an_object_on_with_modifiers_is_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<div v-on.once="handlers"></div>"#),
        EmitError::Unsupported
    );
}

#[test]
fn a_lone_object_on_uses_to_handlers() {
    assert_eq!(
        assembled(r#"<div v-on="handlers"></div>"#),
        "\
const { toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", _toHandlers(handlers, true), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn duplicate_click_handlers_are_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<div @click="a" @click="b"></div>"#),
        EmitError::Unsupported
    );
}
