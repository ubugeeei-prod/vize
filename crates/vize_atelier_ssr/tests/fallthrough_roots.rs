//! A component's fallthrough attrs reach its single rendered root even when
//! that root sits behind nodes the server renders nothing for: a
//! `<template v-if>`, a `<Transition>` / `<KeepAlive>`, or a patterned-template
//! scope. Each expectation is what `@vue/compiler-ssr` emits for the template.

use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr, compile_ssr_with_options};
use vize_s0::Allocator;

fn render_body(code: &str) -> &str {
    let start = code.find("_attrs) {\n").expect("ssrRender signature") + "_attrs) {\n".len();
    &code[start..code.rfind("\n}").expect("ssrRender end")]
}

fn compile(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr(&allocator, source);
    assert!(errors.is_empty(), "{errors:?}");
    render_body(&result.code).into()
}

#[test]
fn a_template_branch_root_inherits_attrs() {
    assert_eq!(
        compile(r#"<template v-if="ok"><p>one</p></template>"#),
        "  if (_ctx.ok) {\n    _push(`<p${_ssrRenderAttrs(_attrs)}>one</p>`)\n  } else {\n    _push(`<!---->`)\n  }"
    );
}

#[test]
fn transparent_builtins_render_their_child_as_the_root() {
    for builtin in ["Transition", "KeepAlive", "keep-alive", "BaseTransition"] {
        let source = vize_s0::cstr!(r#"<{builtin}><p v-if="ok">a</p></{builtin}>"#);
        assert_eq!(
            compile(&source),
            "  if (_ctx.ok) {\n    _push(`<p${_ssrRenderAttrs(_attrs)}>a</p>`)\n  } else {\n    _push(`<!---->`)\n  }",
            "{builtin}"
        );
    }
}

#[test]
fn a_patterned_scope_renders_in_place_without_list_markers() {
    let allocator = Allocator::new();
    let options = SsrCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };
    let (_, errors, result) = compile_ssr_with_options(
        &allocator,
        r#"<template v-match="state"><p v-when="{ const text }">{{ text }}</p><i v-when="_">empty</i></template>"#,
        options,
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(
        render_body(&result.code),
        r#"  { const __vize_match_0 = ((__vize_match_0_0) => { a0: { if (__vize_match_0_0 == null) break a0; if (!("text" in Object(__vize_match_0_0))) break a0; const __vize_match_0_1 = __vize_match_0_0["text"]; { const text = __vize_match_0_1; return [0, text]; } } return [1]; })(_ctx.state)
    if (__vize_match_0[0] === 0) {
      { const [, text] = __vize_match_0
        _push(`<p${_ssrRenderAttrs(_attrs)}>${_ssrInterpolate(text)}</p>`)
      }
    } else if (__vize_match_0[0] === 1) {
      _push(`<i${_ssrRenderAttrs(_attrs)}>empty</i>`)
    } else {
      _push(`<!---->`)
    }
  }"#
    );
}
