//! Native `ui.if` emit pins and the shapes this installment refuses.

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
fn a_root_v_if_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div v-if="ok">hello</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: 0 }, \"hello\"))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_root_v_if_else_matches_the_shipped_snapshot() {
    assert_eq!(
        assembled(r#"<div v-if="ok">yes</div><div v-else>no</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: 0 }, \"yes\"))
    : (_openBlock(), _createElementBlock(\"div\", { key: 1 }, \"no\"))
}"
    );
}

#[test]
fn a_static_branch_key_is_quoted() {
    assert_eq!(
        assembled(r#"<div v-if="ok" key="k"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: \"k\" }))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn mixed_text_in_a_v_if_lists_comment_before_text() {
    // Nested static vnode hoist (`_hoisted_1 = /*#__PURE__*/ _createElementVNode`)
    // is a later P2-11 realization; this pin is helper rank only.
    assert_eq!(
        assembled(r#"<div v-if="ok"><span></span> x</div>"#),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode, createTextVNode: _createTextVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: 0 }, [
      _createElementVNode(\"span\"),
      _createTextVNode(\" x\")
    ]))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn a_component_v_if_is_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<MyComp v-if="ok"></MyComp>"#),
        EmitError::Unsupported
    );
}

#[test]
fn a_template_fragment_v_if_is_unsupported_this_installment() {
    assert_eq!(
        refused(r#"<template v-if="ok"><span></span><span></span></template>"#),
        EmitError::Unsupported
    );
}
