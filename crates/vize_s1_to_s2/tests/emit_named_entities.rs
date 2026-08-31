//! Named HTML entity parity for S2 DOM text and static props.

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
fn named_entity_text_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled("<button>&times;</button>"),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", null, \"\u{00d7}\"))
}"
    );
}

#[test]
fn named_entity_static_attr_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<button title="&times;">close</button>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

const _hoisted_1 = { title: \"\u{00d7}\" }

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", _hoisted_1, \"close\"))
}"
    );
}
