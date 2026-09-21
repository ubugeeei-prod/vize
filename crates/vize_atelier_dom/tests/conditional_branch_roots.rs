//! The vnode a conditional branch renders as its block root. Each expectation
//! is `@vue/compiler-dom`'s output for the same template, except for the
//! patterned-template scope, which follows the RFC 823 reference lowering.

use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_s0::{Allocator, String};

fn render(source: &str, options: DomCompilerOptions) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_options(&allocator, source, options);
    assert!(errors.is_empty(), "{errors:?}");
    result.code
}

/// A `<template v-if>` around one static element renders that element with
/// the branch key. Hoisting it left a keyed fragment around a static vnode,
/// which at a component root also drops attribute fallthrough.
#[test]
fn a_nested_template_branch_renders_its_single_element_as_the_block() {
    assert_eq!(
        render(
            r#"<section><template v-if="ok"><p>one</p></template><template v-else><p>two</p></template></section>"#,
            DomCompilerOptions::default(),
        ),
        r#"function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("section", null, [
    (ok)
      ? (_openBlock(), _createElementBlock("p", { key: 0 }, "one"))
      : (_openBlock(), _createElementBlock("p", { key: 1 }, "two"))
  ]))
}"#
    );
}

/// The root under `<template v-if>` hoists what a `v-if` element hoists: its
/// static descendants. Its own props carry the branch key and stay inline, so
/// no unused `_hoisted_n` copy of them is left in the module.
#[test]
fn a_template_branch_root_hoists_its_static_descendants_only() {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<section><template v-if="ok"><div flex gap-1><i class="x"></i><span>{{ v }}</span></div></template></section>"#,
        DomCompilerOptions::default(),
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(
        result.preamble,
        r#"const { toDisplayString: _toDisplayString, createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock, createCommentVNode: _createCommentVNode } = Vue

const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "x" })
"#
    );
    assert_eq!(
        result.code,
        r#"function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("section", null, [
    (ok)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        flex: "",
        "gap-1": ""
      }, [
        _hoisted_1,
        _createElementVNode("span", null, _toDisplayString(v), 1 /* TEXT */)
      ]))
      : _createCommentVNode("v-if", true)
  ]))
}"#
    );
}

/// `v-once` on the `v-if` element caches the whole chain, as Vue does.
#[test]
fn v_once_on_a_conditional_caches_the_chain() {
    assert_eq!(
        render(
            r#"<section><input v-if="ok" v-once :value="v"/></section>"#,
            DomCompilerOptions::default(),
        ),
        r#"function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("section", null, [
    _cache[0] || (
      _setBlockTracking(-1, true),
      (_cache[0] = (ok)
        ? (_openBlock(), _createElementBlock("input", {
          key: 0,
          value: v
        }, null, 8 /* PROPS */, ["value"]))
        : _createCommentVNode("v-if", true)).cacheIndex = 0,
      _setBlockTracking(1),
      _cache[0]
    )
  ]))
}"#
    );
}

/// A match is a lexical scope, not a list: the selected arm is the block
/// root, so no fragment is rendered and a single-root match keeps fallthrough.
#[test]
fn a_patterned_match_renders_the_selected_arm_as_the_block() {
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };
    assert_eq!(
        render(
            r#"<template v-match="state"><p v-when="{ const text }" :key="text">{{ text }}</p><i v-when="_">empty</i></template>"#,
            options,
        ),
        r#"function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (() => { const __vize_match_0 = ((__vize_match_0_0) => { a0: { if (__vize_match_0_0 == null) break a0; if (!("text" in Object(__vize_match_0_0))) break a0; const __vize_match_0_1 = __vize_match_0_0["text"]; { const text = __vize_match_0_1; return [0, text]; } } return [1]; })(state); return (__vize_match_0[0] === 0)
    ? (() => { const [, text] = __vize_match_0; return (_openBlock(), _createElementBlock("p", { key: text }, _toDisplayString(text), 1 /* TEXT */)) })()
    : (__vize_match_0[0] === 1)
      ? (_openBlock(), _createElementBlock("i", { key: 1 }, "empty"))
    : _createCommentVNode("v-if", true) })()
}"#
    );
}
