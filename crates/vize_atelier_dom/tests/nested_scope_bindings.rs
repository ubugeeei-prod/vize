//! A scope that reuses the name of an enclosing scope variable must not
//! unregister the enclosing one when it closes: what follows the inner scope
//! still reads the outer variable, never `_ctx.<name>`. Dynamic arguments are
//! the props that resolve their identifier at code generation.

use vize_atelier_core::options::CodegenMode;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_s0::{Allocator, String};

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_options(
        &allocator,
        source,
        DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            experimental_patterned_template: true,
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    result.code
}

/// `@vue/compiler-dom` emits `[x || ""]: x` for both elements.
#[test]
fn an_outer_loop_alias_survives_an_inner_loop_that_reuses_its_name() {
    assert_eq!(
        render(
            r#"<div v-for="x in outer"><i v-for="x in inner" :[x]="x"></i><b :[x]="x"></b></div>"#
        ),
        r#"export function render(_ctx, _cache) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.outer, (x) => {
    return (_openBlock(), _createElementBlock("div", null, [
      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.inner, (x) => {
        return (_openBlock(), _createElementBlock("i", { [x || ""]: x }, null, 16 /* FULL_PROPS */))
      }), 256 /* UNKEYED_FRAGMENT */)),
      _createElementVNode("b", _normalizeProps({ [x || ""]: x }), null, 16 /* FULL_PROPS */)
    ]))
  }), 256 /* UNKEYED_FRAGMENT */))
}"#
    );
}

#[test]
fn an_outer_arm_binding_survives_an_inner_match_that_reuses_its_name() {
    assert_eq!(
        render(
            r#"<template v-match="a"><div v-when="{ const x }"><template v-match="b"><i v-when="{ const x }" :[x]="x"></i></template><b :[x]="x"></b></div></template>"#
        ),
        r#"export function render(_ctx, _cache) {
  return (() => { const __vize_match_0 = ((__vize_match_0_0) => { a0: { if (__vize_match_0_0 == null) break a0; if (!("x" in Object(__vize_match_0_0))) break a0; const __vize_match_0_1 = __vize_match_0_0["x"]; { const x = __vize_match_0_1; return [0, x]; } } return [-1]; })(_ctx.a); return (__vize_match_0[0] === 0)
    ? (() => { const [, x] = __vize_match_0; return (_openBlock(), _createElementBlock("div", { key: 0 }, [
      (() => { const __vize_match_1 = ((__vize_match_1_0) => { a0: { if (__vize_match_1_0 == null) break a0; if (!("x" in Object(__vize_match_1_0))) break a0; const __vize_match_1_1 = __vize_match_1_0["x"]; { const x = __vize_match_1_1; return [0, x]; } } return [-1]; })(_ctx.b); return (__vize_match_1[0] === 0)
        ? (() => { const [, x] = __vize_match_1; return (_openBlock(), _createElementBlock("i", {
          key: 0,
          [x || ""]: x
        }, null, 16 /* FULL_PROPS */)) })()
        : _createCommentVNode("v-if", true) })(),
      _createElementVNode("b", _normalizeProps({ [x || ""]: x }), null, 16 /* FULL_PROPS */)
    ])) })()
    : _createCommentVNode("v-if", true) })()
}"#
    );
}
