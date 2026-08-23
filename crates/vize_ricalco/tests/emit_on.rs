//! Static-name `ui.on` emit pins and the shapes this installment refuses.

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
fn a_click_modifier_is_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<div @click.stop="handler"></div>"#),
        EmitError::Unsupported
    );
}

#[test]
fn a_von_object_spread_is_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<div v-on="handlers"></div>"#),
        EmitError::Unsupported
    );
}

#[test]
fn duplicate_click_handlers_are_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<div @click="a" @click="b"></div>"#),
        EmitError::Unsupported
    );
}
