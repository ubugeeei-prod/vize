//! P2-11 installment 1: static native HTML elements emit the same
//! render function the shipped DOM lane does.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_carton::Allocator;
use vize_ricalco::{EmitError, emit_dom, emit_dom_source};

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, _facts, _budget| {
        emit_dom(lowered)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

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
fn static_attributes_are_unsupported_this_installment() {
    with_transformed(r#"<div class="x"></div>"#, |lowered, _, _, _| {
        assert_eq!(emit_dom(lowered), Err(EmitError::Unsupported));
    });
}

#[test]
fn a_component_root_is_unsupported_this_installment() {
    with_transformed("<MyComp/>", |lowered, _, _, _| {
        assert_eq!(emit_dom(lowered), Err(EmitError::Unsupported));
    });
}
