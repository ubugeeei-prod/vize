//! P2-11: static-name `v-bind` modifiers in the S2 DOM emitter.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_ricalco::emit_dom;

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

#[test]
fn a_static_camel_bind_uses_the_camelized_prop_key() {
    assert_eq!(
        assembled(r#"<div :foo-bar.camel="value"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { fooBar: value }, null, 8 /* PROPS */, [\"fooBar\"]))
}"
    );
}

#[test]
fn a_static_prop_bind_prefixes_the_prop_key_and_sets_hydration() {
    assert_eq!(
        assembled(r#"<div :value.prop="value"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { \".value\": value }, null, 40 /* PROPS, NEED_HYDRATION */, [\".value\"]))
}"
    );
}

#[test]
fn a_static_attr_bind_prefixes_the_attr_key() {
    assert_eq!(
        assembled(r#"<div :value.attr="value"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { \"^value\": value }, null, 8 /* PROPS */, [\"^value\"]))
}"
    );
}

#[test]
fn the_dot_shorthand_is_the_same_static_prop_modifier() {
    assert_eq!(
        assembled(r#"<div .value="value"></div>"#),
        assembled(r#"<div :value.prop="value"></div>"#)
    );
}
