//! Ordinary template comments in the S2 DOM emitter.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_s0::Allocator;
use vize_s1_to_s2::{DomEmitOptions, LegacyCaps, emit_dom_source, emit_dom_source_with_options};

fn assembled_with_comments(source: &str, comments: bool) -> String {
    let allocator = Allocator::new();
    emit_dom_source_with_options(
        &allocator,
        source,
        LegacyCaps::VUE3,
        &DomEmitOptions {
            comments,
            ..DomEmitOptions::DEFAULT
        },
    )
    .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
    .assembled()
    .to_string()
}

fn assembled_with_options(source: &str, options: DomEmitOptions<'_>) -> String {
    let allocator = Allocator::new();
    emit_dom_source_with_options(&allocator, source, LegacyCaps::VUE3, &options)
        .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
        .assembled()
        .to_string()
}

fn pin(visual: &str) -> String {
    visual.replace(")\n\n  return", ")\n  \n  return")
}

#[test]
fn ordinary_template_comments_emit_comment_vnodes_when_enabled() {
    assert_eq!(
        assembled_with_comments("<div><!--kept--><span>ok</span></div>", true),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createCommentVNode(\"kept\"),
    _createElementVNode(\"span\", null, \"ok\")
  ]))
}"
    );
}

#[test]
fn ordinary_template_comments_stay_dropped_by_default() {
    let allocator = Allocator::new();
    assert_eq!(
        emit_dom_source(&allocator, "<div><!--kept--><span>ok</span></div>")
            .unwrap_or_else(|error| panic!("emit refused default comments=false case: {error:?}"))
            .assembled()
            .to_string(),
        "\
const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, [
    _createElementVNode(\"span\", null, \"ok\")
  ]))
}"
    );
}

#[test]
fn in_tag_comments_are_parse_options_not_comment_vnodes() {
    let with_in_tag_comment = assembled_with_options(
        "<div // keep the parse extension covered\n  id=\"x\">{{ msg }}</div>",
        DomEmitOptions {
            experimental_in_tag_comments: true,
            ..DomEmitOptions::DEFAULT
        },
    );
    let without_in_tag_comment =
        assembled_with_options("<div id=\"x\">{{ msg }}</div>", Default::default());

    assert_eq!(with_in_tag_comment, without_in_tag_comment);
}

#[test]
fn component_default_slots_preserve_comment_children_when_enabled() {
    assert_eq!(
        assembled_with_comments("<Foo><!--slot--><span>ok</span></Foo>", true),
        pin("\
const { resolveComponent: _resolveComponent, createElementVNode: _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, createCommentVNode: _createCommentVNode, withCtx: _withCtx } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\", null, \"ok\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return (_openBlock(), _createBlock(_component_Foo, null, {
    default: _withCtx(() => [
      _createCommentVNode(\"slot\"),
      _hoisted_1
    ]),
    _: 1 /* STABLE */
  }))
}")
    );
}

#[test]
fn slot_outlet_fallbacks_preserve_comment_children_when_enabled() {
    assert_eq!(
        assembled_with_comments("<slot><!--fallback--><span>ok</span></slot>", true),
        "\
const { renderSlot: _renderSlot, createElementVNode: _createElementVNode, createCommentVNode: _createCommentVNode } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"span\", null, \"ok\")

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _renderSlot(_ctx.$slots, \"default\", {}, () => [
    _createCommentVNode(\"fallback\"),
    _hoisted_1
  ])
}"
    );
}
