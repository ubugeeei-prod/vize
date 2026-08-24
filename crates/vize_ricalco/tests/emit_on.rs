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

/// Vue's extra `newline()` after `genAssets` leaves indent on the blank line.
fn pin(visual: &str) -> String {
    visual.replace(")\n\n  return", ")\n  \n  return")
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

// v-on-storage-synthetic:start
#[test]
fn two_modifiers_per_bucket_keep_the_authored_output() {
    assert_eq!(
        assembled(r#"<div @keyup.capture.once.stop.prevent.enter.esc="handler"></div>"#),
        "\
const { withKeys: _withKeys, withModifiers: _withModifiers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onKeyupCaptureOnce: _withKeys(_withModifiers(handler, [\"stop\",\"prevent\"]), [\"enter\",\"esc\"])
  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onKeyupCaptureOnce\"]))
}"
    );
}

#[test]
fn spilled_modifier_buckets_keep_the_authored_output() {
    assert_eq!(
        assembled(
            r#"<div @keyup.capture.once.passive.stop.prevent.self.enter.esc.space="handler"></div>"#,
        ),
        "\
const { withKeys: _withKeys, withModifiers: _withModifiers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onKeyupCaptureOncePassive: _withKeys(_withModifiers(handler, [\"stop\",\"prevent\",\"self\"]), [\"enter\",\"esc\",\"space\"])
  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onKeyupCaptureOncePassive\"]))
}"
    );
}
// v-on-storage-synthetic:end

#[test]
fn a_click_native_strips_the_modifier_but_keeps_need_hydration() {
    assert_eq!(
        assembled(r#"<div @click.native="handler"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: handler
  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onClick\"]))
}"
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
fn duplicate_click_handlers_merge_into_an_array() {
    assert_eq!(
        assembled(r#"<div @click="a" @click="b"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: [a, b]
  }, null, 8 /* PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn a_colon_event_on_a_component_camelizes() {
    assert_eq!(
        assembled(r#"<Foo @update:modelValue="h" />"#),
        pin("\
const { resolveComponent: _resolveComponent, openBlock: _openBlock, createBlock: _createBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, { \"onUpdate:modelValue\": h }, null, 8 /* PROPS */, [\"onUpdate:modelValue\"]))
}")
    );
}

#[test]
fn a_colon_event_on_a_plain_element_preserves_case() {
    assert_eq!(
        assembled(r#"<div @update:modelValue="h"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { \"on:update:modelValue\": h }, null, 40 /* PROPS, NEED_HYDRATION */, [\"on:update:modelValue\"]))
}"
    );
}

#[test]
fn a_vue_hook_rewrites_to_on_vnode() {
    assert_eq!(
        assembled(r#"<div @vue:mounted="h"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { onVnodeMounted: h }, null, 8 /* PROPS */, [\"onVnodeMounted\"]))
}"
    );
}

#[test]
fn click_and_click_ctrl_merge_on_the_same_key() {
    assert_eq!(
        assembled(r#"<div @click="a" @click.ctrl="b"></div>"#),
        "\
const { withModifiers: _withModifiers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: [a, _withModifiers(b, [\"ctrl\"])]
  }, null, 8 /* PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn click_and_click_once_keep_distinct_keys() {
    assert_eq!(
        assembled(r#"<div @click="a" @click.once="b"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: a,
    onClickOnce: b
  }, null, 40 /* PROPS, NEED_HYDRATION */, [\"onClick\", \"onClickOnce\"]))
}"
    );
}
