//! #11594: Vue's `ssrProcessTransition` drops the direct comment children of a
//! `<Transition>` (vuejs/core#5351), renders its children without nested
//! fragments, and wraps an `appear` transition in `<template>`. The client's
//! `BaseTransition` hydrates exactly that. `<KeepAlive>` and `<BaseTransition>`
//! keep Vue's default child rules.
#![cfg(feature = "legacy-differential")]

use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// Template, then the render function. Its body is what Vue 3.5.44's
/// compiler-ssr emits with `comments: true` (the Vite and Nuxt development
/// option).
const CASES: &[(&str, &str, &str)] = &[
    (
        "transition-comment",
        r#"<div><Transition><!-- c --><p>hello</p></Transition></div>"#,
        "function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><p>hello</p></div>`)
}",
    ),
    (
        "transition-appear",
        r#"<div><Transition appear><!-- c --><p>hello</p></Transition></div>"#,
        "function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><template><p>hello</p></template></div>`)
}",
    ),
    (
        "transition-bound-appear",
        r#"<div><Transition :appear="ok"><p>hello</p></Transition></div>"#,
        "function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><template><p>hello</p></template></div>`)
}",
    ),
    (
        "transition-branch-fragment",
        r#"<div><Transition><template v-if="ok"><p>a</p><p>b</p></template></Transition></div>"#,
        "function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}>`)
  if (_ctx.ok) {
    _push(`<p>a</p><p>b</p>`)
  } else {
    _push(`<!---->`)
  }
  _push(`</div>`)
}",
    ),
    (
        "transition-loop",
        r#"<div><Transition><p v-for="i in 2" :key="i">a</p></Transition></div>"#,
        "function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}>`)
  _ssrRenderList(2, (i) => {
    _push(`<p>a</p>`)
  })
  _push(`</div>`)
}",
    ),
    // Negative controls: Vue keeps these comments.
    (
        "keep-alive-comment",
        r#"<div><KeepAlive><!-- c --><p>hello</p></KeepAlive></div>"#,
        "function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><!-- c --><p>hello</p></div>`)
}",
    ),
    (
        "base-transition-comment",
        r#"<div><BaseTransition><!-- c --><p>hello</p></BaseTransition></div>"#,
        "function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><!-- c --><p>hello</p></div>`)
}",
    ),
    (
        "element-comment",
        r#"<div><!-- c --><p>hello</p></div>"#,
        "function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<div${_ssrRenderAttrs(_attrs)}><!-- c --><p>hello</p></div>`)
}",
    ),
];

#[test]
fn transition_children_follow_vue_ssr_rules_in_both_lanes() {
    for &(name, source, expected) in CASES {
        let compiler_options = SsrCompilerOptions {
            comments: true,
            ..Default::default()
        };
        let pair = vize_atelier_ssr::differential::compare_ssr_lanes(
            source,
            &compiler_options,
            &SsrCompilerExperimentalOptions::default(),
        );
        assert!(pair.selected.errors.is_empty(), "{name}");
        assert!(pair.legacy.errors.is_empty(), "{name}");
        assert_eq!(pair.selected.result.code.trim_end(), expected, "{name}");
        assert_eq!(pair.legacy.result.code.trim_end(), expected, "{name}");
    }
}
