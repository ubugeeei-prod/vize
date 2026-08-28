//! `vue.memo` (`v-memo`) emission pins.

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

/// Vue's extra `newline()` after `genAssets` leaves indent on the blank line.
fn pin(visual: &str) -> String {
    visual.replace(")\n\n  return", ")\n  \n  return")
}

#[test]
fn native_v_memo_wraps_the_block_vnode_in_with_memo() {
    assert_eq!(
        assembled(r#"<div v-memo="[id]">x</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode, withMemo: _withMemo } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _withMemo([id], () => (_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(\"x\")
  ])), _cache, 0)
}"
    );
}

#[test]
fn native_v_memo_forces_interpolation_children_into_text_vnodes() {
    assert_eq!(
        assembled(r#"<div v-memo="[id]">{{ msg }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode, withMemo: _withMemo } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _withMemo([id], () => (_openBlock(), _createElementBlock(\"div\", null, [
    _createTextVNode(_toDisplayString(msg), 1 /* TEXT */)
  ])), _cache, 0)
}"
    );
}

#[test]
fn native_v_memo_keeps_dynamic_props_array_without_a_numeric_patch_flag() {
    assert_eq!(
        assembled(r#"<div v-memo="[id]" :id="id">{{ msg }}</div>"#),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, createTextVNode: _createTextVNode, withMemo: _withMemo } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return _withMemo([id], () => (_openBlock(), _createElementBlock(\"div\", { id: id }, [
    _createTextVNode(_toDisplayString(msg), 1 /* TEXT */)
  ], [\"id\"])), _cache, 0)
}"
    );
}

#[test]
fn component_v_memo_uses_create_vnode_inside_the_cache_wrapper() {
    assert_eq!(
        assembled(r#"<Foo v-memo="[prop]" :prop="prop" />"#),
        pin("\
const { resolveComponent: _resolveComponent, createVNode: _createVNode, withMemo: _withMemo } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Foo = _resolveComponent(\"Foo\")

  return _withMemo([prop], () => _createVNode(_component_Foo, { prop: prop }, null, 8 /* PROPS */, [\"prop\"]), _cache, 0)
}")
    );
}

#[test]
fn v_if_branch_v_memo_stays_inert_like_the_shipped_lane() {
    assert_eq!(
        assembled(r#"<div v-if="ok" v-memo="[id]">x</div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (ok)
    ? (_openBlock(), _createElementBlock(\"div\", { key: 0 }, \"x\"))
    : _createCommentVNode(\"v-if\", true)
}"
    );
}

#[test]
fn v_for_v_memo_emits_the_cached_item_reuse_guard() {
    assert_eq!(
        assembled(
            r#"<div v-for="item in items" :key="item.id" v-memo="[item.selected]">{{ item.name }}</div>"#
        ),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createTextVNode: _createTextVNode, renderList: _renderList, withMemo: _withMemo, isMemoSame: _isMemoSame } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item, __, ___, _cached) => {
    const _memo = ([item.selected])
    if (_cached && _cached.el && _cached.key === item.id && _isMemoSame(_cached, _memo)) return _cached
    const _item = (_openBlock(), _createElementBlock(\"div\", { key: item.id }, [
      _createTextVNode(_toDisplayString(item.name), 1 /* TEXT */)
    ]))
    _item.memo = _memo
    return _item
  }, _cache, 0), 128 /* KEYED_FRAGMENT */))
}"
    );
}

#[test]
fn opaque_memo_expression_has_a_typed_refusal() {
    assert_eq!(
        refused(r#"<div v-memo="%">x</div>"#).reason(),
        Some(Reason::MemoExpressionNotJs)
    );
}
