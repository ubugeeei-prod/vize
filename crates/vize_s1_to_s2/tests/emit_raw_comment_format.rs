//! Raw JS comment formatting parity for emitted prop expressions.

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
fn object_prop_value_line_comment_is_emitted_as_block_comment() {
    assert_eq!(
        assembled(
            r#"<div :data-payload="{
  value,
  next: count // payload lane note
}"></div>"#,
        ),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { \"data-payload\": {
  value,
  next: count /*  payload lane note */
} }, null, 8 /* PROPS */, [\"data-payload\"]))
}"
    );
}

#[test]
fn normalized_style_line_comment_is_emitted_as_block_comment() {
    assert_eq!(
        assembled(
            r#"<div :style="{
  transitionDelay: `${index * 50}ms`, // delay between each item
}"></div>"#,
        ),
        "\
const { normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    style: _normalizeStyle({
  transitionDelay: `${index * 50}ms`, /*  delay between each item */
})
  }, null, 4 /* STYLE */))
}"
    );
}

#[test]
fn normalized_class_line_comment_is_emitted_as_block_comment() {
    assert_eq!(
        assembled(
            r#"<div class="base" :class="[
  active,
  pending // pending class note
]"></div>"#,
        ),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass([\"base\", [
  active,
  pending /*  pending class note */
]])
  }, null, 2 /* CLASS */))
}"
    );
}

#[test]
fn object_on_spread_line_comment_is_emitted_as_block_comment() {
    assert_eq!(
        assembled(
            r#"<button v-on="{
  click: onClick // listener map note
}"></button>"#,
        ),
        "\
const { toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", _toHandlers({
  click: onClick /*  listener map note */
}, true), null, 16 /* FULL_PROPS */))
}"
    );
}
